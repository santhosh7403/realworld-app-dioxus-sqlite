#[cfg(feature = "server")]
use argon2::{password_hash::PasswordVerifier, Argon2};
use dioxus::prelude::*;

#[derive(serde::Deserialize, Clone, serde::Serialize)]
pub enum SignupResponse {
    ValidationError(String),
    CreateUserError(String),
    Success,
}

#[tracing::instrument]
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

#[tracing::instrument]
#[server]
pub async fn signup_action(
    username: String,
    email: String,
    password: String,
) -> Result<SignupResponse, ServerFnError> {
    match validate_signup(username.clone(), email, password) {
        Ok(user) => match user.insert().await {
            Ok(_) => {
                let server_context = server_context();
                let mut resp = server_context.response_parts_mut();

                crate::auth::set_username(username, &mut resp.headers).await;
                Ok(SignupResponse::Success)
            }
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

#[server]
#[tracing::instrument]
pub async fn login_action(username: String, password: String) -> Result<String, ServerFnError> {
    let server_context = server_context();
    let mut resp = server_context.response_parts_mut();
    // dioxus_logger::tracing::info!("login triggered in api");

    let hash_pass_row = sqlx::query!("SELECT password FROM Users where username=$1", username)
        .fetch_one(crate::database::server::get_db())
        .await
        .map_err(|err| {
            tracing::debug!("DB err: {}", err);
            resp.status = axum::http::StatusCode::FORBIDDEN;
            // response_options.set_status(axum::http::StatusCode::FORBIDDEN);
            ServerFnError::new("Unsuccessful: User not available".to_string())
        })?;

    let parsed_hash =
        argon2::password_hash::PasswordHash::new(&hash_pass_row.password).map_err(|_| {
            // response_options.set_status(axum::http::StatusCode::FORBIDDEN);
            resp.status = axum::http::StatusCode::FORBIDDEN;
            ServerFnError::new("Unsuccessful: Hash error".to_string())
        })?;

    let argon2 = Argon2::default();
    if argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
    {
        crate::auth::set_username(username, &mut resp.headers).await;
        use crate::server_fn::redirect;
        redirect::call_redirect_hook("/");
        Ok("Successful".to_string())
    } else {
        // response_options.set_status(axum::http::StatusCode::FORBIDDEN);
        resp.status = axum::http::StatusCode::FORBIDDEN;

        Err(ServerFnError::new(
            "Unsuccessful: Password not matching".to_string(),
        ))
    }
}

#[server]
#[tracing::instrument]
pub async fn logout_action() -> Result<(), ServerFnError> {
    let server_context = server_context();
    let mut resp = server_context.response_parts_mut();

    resp.headers.insert(
        axum::http::header::SET_COOKIE,
        axum::http::HeaderValue::from_str(crate::auth::REMOVE_COOKIE)
            .expect("header value couldn't be set"),
    );
    Ok(())
}

#[server]
#[tracing::instrument]
pub async fn current_user() -> Result<crate::models::User, ServerFnError> {
    let server_context = server_context();
    let req: axum::http::request::Parts = server_context.extract().await?;

    let Some(logged_user) = super::get_username(req) else {
        return Err(ServerFnError::ServerError("you must be logged in".into()));
    };
    crate::models::User::get(logged_user).await.map_err(|err| {
        tracing::error!("problem while retrieving current_user: {err:?}");
        ServerFnError::ServerError("you must be logged in".into())
    })
}
