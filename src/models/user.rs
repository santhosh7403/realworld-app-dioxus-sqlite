#[cfg(feature = "server")]
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq)]
pub struct UserPreview {
    pub username: String,
    pub image: Option<String>,
    pub following: bool,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq)]
pub struct User {
    pub username: String,
    // #[cfg_attr(feature = "hydrate", allow(dead_code))]
    #[serde(skip_serializing)]
    password: Option<String>,
    email: String,
    bio: Option<String>,
    image: Option<String>,
}
#[cfg(feature = "server")]
static EMAIL_REGEX: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();

impl User {
    #[inline]
    pub fn username(&self) -> String {
        self.username.to_string()
    }
    #[inline]
    pub fn email(&self) -> String {
        self.email.to_string()
    }
    #[inline]
    pub fn bio(&self) -> Option<String> {
        self.bio.clone()
    }
    #[inline]
    pub fn image(&self) -> Option<String> {
        self.image.clone()
    }

    #[cfg(feature = "server")]
    pub fn set_password(mut self, password: String) -> Result<Self, String> {
        if password.len() < 4 {
            return Err("You need to provide a stronger password".into());
        }
        self.password = Some(password);
        Ok(self)
    }

    #[cfg(feature = "server")]
    pub fn set_username(mut self, username: String) -> Result<Self, String> {
        if username.len() < 4 {
            return Err(format!(
                "Username {username} is too short, at least 4 characters"
            ));
        }
        self.username = username;
        Ok(self)
    }

    #[cfg(feature = "server")]
    fn validate_email(email: &str) -> bool {
        EMAIL_REGEX
            .get_or_init(|| regex::Regex::new(r"^[\w\-\.]+@([\w-]+\.)+\w{2,4}$").unwrap())
            .is_match(email)
    }

    #[cfg(feature = "server")]
    pub fn set_email(mut self, email: String) -> Result<Self, String> {
        if !Self::validate_email(&email) {
            return Err(format!(
                "The email {email} is invalid, provide a correct one"
            ));
        }
        self.email = email;
        Ok(self)
    }

    #[cfg(feature = "server")]
    pub fn set_bio(mut self, bio: String) -> Result<Self, String> {
        static BIO_MIN: usize = 10;
        if bio.is_empty() {
            self.bio = None;
        } else if bio.len() < BIO_MIN {
            return Err("bio too short, at least 10 characters".into());
        } else {
            self.bio = Some(bio);
        }
        Ok(self)
    }

    #[cfg(feature = "server")]
    #[inline]
    pub fn set_image(mut self, image: String) -> Result<Self, String> {
        if image.is_empty() {
            self.image = None;
            // TODO: This is incorrect! changeme in the future for a proper validation
        } else if !image.starts_with("http") {
            return Err("Invalid image url!".into());
        } else {
            self.image = Some(image);
        }
        Ok(self)
    }

    #[cfg(feature = "server")]
    pub async fn get(username: String) -> Result<Self, sqlx::Error> {
        sqlx::query_as!(
            Self,
            "SELECT username, email, bio, image, password FROM users WHERE username=$1",
            username
        )
        .fetch_one(crate::database::server::get_db())
        .await
    }

    #[cfg(feature = "server")]
    pub async fn get_email(email: String) -> Result<Self, sqlx::Error> {
        sqlx::query_as!(
            Self,
            "SELECT username, email, bio, image, password FROM users WHERE email=$1",
            email
        )
        .fetch_one(crate::database::server::get_db())
        .await
    }

    #[cfg(feature = "server")]
    pub async fn insert(&self) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
        // Hash the password
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let hashed_password =
            match argon2.hash_password(self.password.clone().unwrap().as_bytes(), &salt) {
                Ok(hash) => Some(hash.to_string()),
                Err(e) => {
                    tracing::error!("Failed to hash password: {:?}", e);
                    return Err(sqlx::Error::InvalidArgument(e.to_string()));
                }
            };

        sqlx::query!(
            "INSERT INTO Users(username, email, password) VALUES ($1, $2, $3)",
            self.username,
            self.email,
            hashed_password,
        )
        .execute(crate::database::server::get_db())
        .await
    }

    #[cfg(feature = "server")]
    pub async fn update(&self) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
        let password_is_some = self.password.is_some();
        let mut hashed_password = None;
        if password_is_some {
            let salt = SaltString::generate(&mut OsRng);
            let argon2 = Argon2::default();
            hashed_password =
                match argon2.hash_password(self.password.clone().unwrap().as_bytes(), &salt) {
                    Ok(hash) => Some(hash.to_string()),
                    Err(e) => {
                        tracing::error!("Failed to hash password: {:?}", e);
                        return Err(sqlx::Error::InvalidArgument(e.to_string()));
                    }
                };
        }
        sqlx::query!(
            "
UPDATE Users SET
    image=$2,
    bio=$3,
    email=$4,
    password=CASE WHEN $5 THEN $6 ELSE password END
WHERE username=$1",
            self.username,
            self.image,
            self.bio,
            self.email,
            password_is_some,
            hashed_password,
        )
        .execute(crate::database::server::get_db())
        .await
    }
}
