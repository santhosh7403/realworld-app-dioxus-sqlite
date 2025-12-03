#[cfg(feature = "server")]
use argon2::{password_hash::PasswordVerifier, Argon2};
use dioxus::fullstack::{SetCookie, SetHeader};
use dioxus::prelude::*;

#[derive(serde::Deserialize, Clone, serde::Serialize)]
pub enum SignupResponse {
    ValidationError(String),
    CreateUserError(String),
    Success,
}

#[cfg(feature = "server")]
use dioxus::fullstack::{Cookie, TypedHeader};

#[cfg(feature = "server")]
pub fn validate_signup(
    username: String,
    email: String,
    password: String,
) -> Result<crate::models::User, String> {
    crate::models::User::default()
        .set_username(username)?
        .set_password(password)?
        .set_email(email)
}

#[server]
pub async fn signup_action(
    username: String,
    email: String,
    password: String,
) -> Result<SignupResponse, ServerFnError> {
    match validate_signup(username.clone(), email, password.clone()) {
        Ok(user) => match user.insert().await {
            Ok(_) => Ok(SignupResponse::Success),
            Err(x) => {
                let x = x.to_string();
                Ok(if x.contains("UNIQUE constraint failed: Users.email") {
                    SignupResponse::CreateUserError("Duplicated email".to_string())
                } else if x.contains("UNIQUE constraint failed: Users.username") {
                    SignupResponse::CreateUserError("Duplicated user".to_string())
                } else {
                    tracing::error!("error from DB: {}", x);
                    SignupResponse::CreateUserError(
                        "There is some problem in user creation, check log".to_string(),
                    )
                })
            }
        },
        Err(x) => Ok(SignupResponse::ValidationError(x)),
    }
}

#[post("/api/logout", header: TypedHeader<Cookie>)]
pub async fn logout() -> Result<SetHeader<SetCookie>> {
    Ok(SetHeader::new(format!(
        "token=; path=/; expires=Thu, 01 Jan 1970 00:00:00 GMT"
    ))?)
}

#[post("/api/login")]
pub async fn login(username: String, password: String) -> ServerFnResult<SetHeader<SetCookie>> {
    let hash_pass_row = match sqlx::query!("SELECT password FROM Users where username=$1", username)
        .fetch_one(crate::database::server::get_db())
        .await
    {
        Ok(row) => row,
        Err(err) => {
            tracing::error!("DB err: {}", err);
            return Err(ServerFnError::ServerError {
                message: "Invalid username or password".to_string(),
                code: 401,
                details: serde_json::json!("Invalid username or password").into(),
            });
        }
    };

    let parsed_hash = match argon2::password_hash::PasswordHash::new(&hash_pass_row.password) {
        Ok(hash) => hash,
        Err(err) => {
            tracing::error!("Failed to hash password: {}", err);
            return Err(ServerFnError::new(
                "Unexpected error occured while login, please try later",
            ));
        }
    };

    let argon2 = Argon2::default();
    if argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
    {
        let token = match crate::auth::encode_token(crate::auth::TokenClaims {
            sub: username,
            exp: (sqlx::types::chrono::Utc::now().timestamp() as usize) + 3_600_000,
        }) {
            Ok(token) => token,
            Err(err) => {
                tracing::error!("Token encode error: {}", err);
                return Err(ServerFnError::new(
                    "Unexpected error occured while login, please try later",
                ));
            }
        };
        let header = match SetHeader::new(format!("token={}; path=/; HttpOnly", token)) {
            Ok(h) => h,
            Err(err) => {
                tracing::error!("failed to construct SetHeader: {}", err);
                return Err(ServerFnError::new(
                    "Unexpected error occured while login, please try later",
                ));
            }
        };
        Ok(header)
    } else {
        Err(ServerFnError::ServerError {
            message: "Invalid username or password".to_string(),
            code: 401,
            details: serde_json::json!("Invalid username or password").into(),
        })
    }
}

#[tracing::instrument]
#[post("/api/current_user", header: TypedHeader<Cookie>)]
pub async fn current_user() -> Result<Option<crate::models::User>, ServerFnError> {
    let Some(logged_user) = super::get_username_from_cookie(header) else {
        return Ok(None);
    };
    Ok(crate::models::User::get(logged_user)
        .await
        .map_err(|err| {
            tracing::error!("problem while retrieving current_user: {err:?}");
            ServerFnError::new("Problem while retrieving current user")
        })
        .ok())
}
