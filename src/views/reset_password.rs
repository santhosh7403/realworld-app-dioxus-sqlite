use dioxus::{document, prelude::*};
use std::env;

use crate::{auth::logout, LoggedInUser};

#[cfg(feature = "server")]
#[allow(dead_code)]
struct EmailCredentials {
    email: String,
    passwd: String,
    smtp_server: String,
}

#[cfg(feature = "server")]
static EMAIL_CREDS: std::sync::OnceLock<EmailCredentials> = std::sync::OnceLock::new();

#[tracing::instrument]
#[post("/api/reset_password_1", headers: dioxus::fullstack::HeaderMap)]
pub async fn reset_password_1(email: String) -> Result<String, ServerFnError> {
    if let Err(x) = crate::models::User::get_email(email.clone()).await {
        let err = format!("Bad email ID: Provided email not found.");
        tracing::error!("{err} {x:?} ");
        return Err(ServerFnError::new(err));
    } else {
        let creds = EMAIL_CREDS.get_or_init(|| EmailCredentials {
            email: env::var("MAILER_EMAIL").unwrap(),
            passwd: env::var("MAILER_PASSWD").unwrap(),
            smtp_server: env::var("MAILER_SMTP_SERVER").unwrap(),
        });
        // dioxus::logger::tracing::info!("the header is {headers:?}");
        let host = headers.get("host").unwrap().to_str().unwrap();
        let scheme = if cfg!(debug_assertions) {
            "http"
        } else {
            "https"
        };
        let token = crate::auth::encode_token(crate::auth::TokenClaims {
            sub: email.clone(),
            exp: (sqlx::types::chrono::Utc::now().timestamp() as usize) + 3_600,
        })
        .unwrap();
        let uri = format!(
            "{}://{}/reset_password?token={}",
            scheme,
            host.to_owned(),
            token
        );
        // Build a simple multipart message
        let message = mail_send::mail_builder::MessageBuilder::new()
            .from(("Realworld Dioxus", creds.email.as_str()))
            .to(vec![("You", email.as_str())])
            .subject("Your password reset from realworld leptos")
            .text_body(format!(
                "You can reset your password accessing the following link: {uri}"
            ));

        // Connect to the SMTP submissions port, upgrade to TLS and
        // authenticate using the provided credentials.
        dioxus::logger::tracing::info!("The email is {:?}", message);

        // ********* UNCOMMENT IF NEEDED *********
        // if smtp available, then uncomment below mail send part. Else use above logging to get a reset link to test
        // Incorrect smtp may cause the thread to panic after multiple attempts

        // mail_send::SmtpClientBuilder::new(creds.smtp_server.as_str(), 587)
        //     .implicit_tls(false)
        //     .credentials((creds.email.as_str(), creds.passwd.as_str()))
        //     .connect()
        //     .await?
        //     .send(message)
        //     .await?
    }
    return Ok(String::from(
        "Email sent. Check email and click the reset url link inside.",
    ));
}

#[tracing::instrument]
#[server]
pub async fn reset_password_2(
    token: String,
    password: String,
    confirm: String,
) -> Result<String, ServerFnError> {
    if !(password == confirm) {
        return Err(ServerFnError::new(
            "Passwords do not match, please retry!".to_string(),
        ));
    }
    let Ok(claims) = crate::auth::decode_token(token.as_str()) else {
        tracing::info!("Invalid token provided");
        return Err(ServerFnError::new("Invalid token provided!".to_string()));
    };
    let email = claims.claims.sub;
    let Ok(user) = crate::models::User::get_email(email.clone()).await else {
        tracing::info!("User does not exist");
        return Err(ServerFnError::new("User does not exist!".to_string()));
    };
    match user.set_password(password) {
        Ok(u) => {
            if let Err(error) = u.update().await {
                tracing::error!(email, ?error, "error while resetting the password");
                return Err(ServerFnError::new(error.to_string()));
            } else {
                // A real password reset would have a list of issued tokens and invalidation over
                // the used ones. As this would grow much bigger in complexity, I prefer to write
                // down this security vulnerability and left it simple :)
                // message = String::from("Password successfully reset, please, proceed to login");
                return Ok("Password successfully changed, please, proceed to login".to_string());
            }
        }
        Err(x) => {
            return Err(ServerFnError::new(x));
        }
    }
}

#[component]
pub fn ResetPasswd(token: ReadSignal<String>) -> Element {
    let mut passwd_visible = use_signal(|| false);
    let mut reset_status = use_signal(|| String::new());
    let mut email = use_signal(|| String::new());
    let mut passwd = use_signal(|| String::new());
    let mut confirm_passwd = use_signal(|| String::new());

    let on_click = move |_| async move {
        if token().is_empty() {
            let reset_res1 = reset_password_1(email()).await;
            match reset_res1 {
                Ok(msg) => reset_status.set(msg),
                Err(err) => reset_status.set(err.to_string()),
            }
        } else {
            let reset_res2 = reset_password_2(token(), passwd(), confirm_passwd()).await;
            match reset_res2 {
                Ok(msg) => {
                    let _ = logout().await;
                    reset_status.set(msg);
                    let mut logged_user = use_context::<Signal<LoggedInUser>>();
                    logged_user.set(LoggedInUser(None));
                    let nav = navigator();
                    nav.replace(crate::Route::Login {});
                }
                Err(err) => reset_status.set(err.to_string()),
            }
        }
    };

    let on_cancel = move |_| {
        let nav = navigator();
        nav.replace(crate::Route::Home {});
    };

    use_effect(move || {
        match (
            passwd().is_empty(),
            confirm_passwd().is_empty(),
            passwd(),
            confirm_passwd(),
        ) {
            (false, false, p1, p2) if p1 != p2 => {
                reset_status.set("Passwords do not match!".to_string())
            }
            (false, false, p1, p2) if p1 == p2 => {
                reset_status.set("Passwords matched!".to_string())
            }
            _ => (),
        }
    });

    rsx! {
        div { class: "fixed inset-0 z-50 flex items-center justify-center bg-gray-900 bg-opacity-60",
            document::Title { "Reset Password" }
            div { class: "block rounded-lg bg-white w-2/5 p-4 shadow-[0_2px_15px_-3px_rgba(0,0,0,0.07),0_10px_20px_-2px_rgba(0,0,0,0.04)] z-70",
                h5 { class: "mb-5 text-xl font-medium leading-tight text-neutral-800",
                    "Reset Password."
                }

                if !token().is_empty() {
                    label {
                        class: "block text-gray-700 text-sm font-bold mb-2",
                        r#for: "password",
                        "Set a new password."
                    }
                    div { class: "mb-5 relative",
                        input {
                            name: "password",
                            class: "shadow appearance-none rounded w-full py-2 px-3 text-gray-700 leading-tight focus:ring",
                            r#type: format!("{}", if passwd_visible() { "text" } else { "password" }),
                            placeholder: "New Password",
                            oninput: move |evt| passwd.set(evt.value()),
                        }
                        span {
                            class: "absolute inset-y-0 right-0 flex items-center pr-3 cursor-pointer  text-gray-500",
                            onclick: move |_| passwd_visible.toggle(),
                            i { class: format!("{}", if passwd_visible() { "far fa-eye" } else { "far fa-eye-slash" }) }
                        }
                    }

                    div { class: "mb-5 relative",
                        input {
                            name: "confirm_password",
                            class: "shadow appearance-none rounded w-full py-2 px-3 text-gray-700 leading-tight focus:ring",
                            r#type: format!("{}", if passwd_visible() { "text" } else { "password" }),
                            placeholder: "Confirm Password",
                            oninput: move |evt| confirm_passwd.set(evt.value()),
                        }
                        span {
                            class: "absolute inset-y-0 right-0 flex items-center pr-3 cursor-pointer  text-gray-500",
                            onclick: move |_| passwd_visible.toggle(),
                            i { class: format!("{}", if passwd_visible() { "far fa-eye" } else { "far fa-eye-slash" }) }
                        }
                    }
                } else {
                    div { class: "mb-5",
                        label {
                            class: "block text-gray-700 text-sm font-bold mb-2",
                            r#for: "email",
                            "Provide your linked email address with your user account."
                            input {
                                class: "shadow appearance-none rounded w-full py-2 px-3 text-gray-700 leading-tight focus:ring",
                                id: "email",
                                name: "email",
                                r#type: "text",
                                placeholder: "Registered email",
                                required: true,
                                oninput: move |evt| email.set(evt.value()),
                            }
                        }
                    }
                }

                div { class: "mb-5",
                    p {
                        class: format!(
                            "font-medium {}",
                            if reset_status().starts_with("Email sent")
                                || reset_status().starts_with("Password successfully changed")
                                || reset_status().starts_with("Passwords matched!")
                            {
                                "text-green-500"
                            } else {
                                "text-red-500"
                            },
                        ),

                        strong { {reset_status()} }
                    }
                }
                div { class: "flex justify-between mb-5",
                    button {
                        class: "bg-blue-700 hover:bg-blue-800 px-5 py-3 text-white rounded-lg disabled:bg-gray-400 disabled:cursor-not-allowed focus:outline-none focus:ring-2 focus:ring-blue-700",
                        r#type: "button",
                        disabled: match (
                            token().is_empty(),
                            email().is_empty(),
                            passwd().is_empty(),
                            confirm_passwd().is_empty(),
                            passwd() == confirm_passwd(),
                            reset_status().starts_with("Password successfully changed"),
                            reset_status().starts_with("Email sent."),
                        ) {
                            (true, false, _, _, _, _, false) => false,
                            (true, false, _, _, _, _, true) => true,
                            (false, _, false, false, true, true, _) => true,
                            (false, _, false, false, true, false, _) => false,
                            _ => true,
                        },
                        onclick: on_click,
                        {
                            format!(
                                "{}",
                                if token().is_empty() {
                                    "Send Reset link in email"
                                } else {
                                    "Reset Password & Logout"
                                },
                            )
                        }
                    }
                    button {
                        class: "bg-blue-700 hover:bg-blue-800 px-5 py-3 text-white rounded-lg disabled:bg-gray-400 disabled:cursor-not-allowed focus:outline-none focus:ring-2 focus:ring-blue-700",
                        onclick: on_cancel,
                        "Cancel"
                    }
                }
            }
        }
    }
}
