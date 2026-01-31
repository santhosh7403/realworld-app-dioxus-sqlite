use dioxus::{document, prelude::*};

#[cfg(feature = "server")]
use dioxus::fullstack::{Cookie, TypedHeader};

use crate::{auth::logout, LoggedInUser};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum SettingsUpdateError {
    PasswordsNotMatch,
    Successful,
    ValidationError(String),
}

#[tracing::instrument]
#[post("/api/settings_update", header: TypedHeader<Cookie>)]
pub async fn settings_update(
    image: String,
    bio: String,
    email: String,
    password: String,
    confirm_password: String,
) -> Result<SettingsUpdateError, ServerFnError> {
    let user = get_user(header).await?;
    let username = user.username();
    let user = match update_user_validation(user, image, bio, email, password, &confirm_password) {
        Ok(x) => x,
        Err(x) => return Ok(x),
    };
    user.update()
        .await
        .map(|_| SettingsUpdateError::Successful)
        .map_err(move |x| {
            tracing::error!(
                "Problem while updating user: {} with error {}",
                username,
                x.to_string()
            );
            ServerFnError::new("Problem while updating user")
        })
}

#[cfg(feature = "server")]
fn update_user_validation(
    mut user: crate::models::User,
    image: String,
    bio: String,
    email: String,
    password: String,
    confirm_password: &str,
) -> Result<crate::models::User, SettingsUpdateError> {
    if !password.is_empty() {
        if password != confirm_password {
            return Err(SettingsUpdateError::PasswordsNotMatch);
        }
        user = user
            .set_password(password)
            .map_err(SettingsUpdateError::ValidationError)?;
    }

    user.set_email(email)
        .map_err(SettingsUpdateError::ValidationError)?
        .set_bio(bio)
        .map_err(SettingsUpdateError::ValidationError)?
        .set_image(image)
        .map_err(SettingsUpdateError::ValidationError)
}

#[cfg(feature = "server")]
async fn get_user(header: TypedHeader<Cookie>) -> Result<crate::models::User, ServerFnError> {
    let Some(username) = crate::auth::get_username_from_cookie(header) else {
        return Err(ServerFnError::new(
            "You need to be authenticated".to_string(),
        ));
    };

    crate::models::User::get(username).await.map_err(|x| {
        let err = x.to_string();
        tracing::error!("problem while getting the user {err}");
        ServerFnError::new(err)
    })
}

#[get("/api/settings_get", header: TypedHeader<Cookie>)]
pub async fn settings_get() -> Result<crate::models::User, ServerFnError> {
    get_user(header).await
}

#[component]
pub fn Settings() -> Element {
    let mut user_settings = use_signal(|| crate::models::User::default());
    let mut passwd_visible = use_signal(|| false);
    let mut update_status = use_signal(|| String::new());
    let mut image_url = use_signal(|| String::new());
    let mut no_image_url_yet = use_signal(|| true);
    let mut bio = use_signal(|| String::new());
    let mut no_bio_yet = use_signal(|| true);
    let mut email = use_signal(|| String::new());
    let mut no_email_yet = use_signal(|| true);
    let mut passwd = use_signal(|| String::new());
    let mut confirm_passwd = use_signal(|| String::new());
    let mut is_passwd_change = use_signal(|| false);

    let mut settings_fut = use_resource(move || async move {
        match settings_get().await {
            Ok(user) => user_settings.set(user),
            Err(_) => (),
        }
    });

    let on_update = move |_| async move {
        update_status.set(String::new());
        let email = match (email().is_empty(), no_email_yet()) {
            (false, false) => email(),
            (_, true) => user_settings().email(),
            _ => "".to_string(),
        };
        let bio = match (
            bio().is_empty(),
            user_settings().bio().unwrap_or_default().is_empty(),
            no_bio_yet(),
        ) {
            (false, _, false) => bio(),
            (_, false, true) => user_settings().bio().unwrap(),
            _ => "".to_string(),
        };

        let image = match (
            image_url().is_empty(),
            user_settings().image().is_some(),
            no_image_url_yet(),
        ) {
            (false, _, false) => Some(image_url()),
            (_, true, true) => user_settings().image(),
            _ => Some("".to_string()),
        };
        let update_result = settings_update(
            image.unwrap_or_default(),
            bio,
            email,
            passwd(),
            confirm_passwd(),
        )
        .await;

        match update_result {
            Ok(SettingsUpdateError::Successful) => {
                update_status.set("Successful.".to_string());
                if is_passwd_change() {
                    let mut logged_user = use_context::<Signal<LoggedInUser>>();
                    logged_user.set(LoggedInUser(None));
                    let _ = logout().await;
                    let nav = navigator();
                    nav.replace(crate::Route::Login {});
                }
                settings_fut.restart();
            }
            Ok(SettingsUpdateError::PasswordsNotMatch) => {
                update_status.set("Error: New password and Confirm password do not match!".into())
            }
            Ok(SettingsUpdateError::ValidationError(err)) => update_status.set(err.to_string()),
            Err(err) => update_status.set(format!("Unexpected error: {err}")),
        }
    };

    let on_cancel = move |_| {
        let nav = navigator();
        if nav.can_go_back() {
            nav.go_back();
        } else {
            nav.replace(crate::Route::Home {});
        }
    };

    let mut passwords_match = move || match (passwd().is_empty(), confirm_passwd().is_empty()) {
        (true, true) => {
            update_status.set(String::new());
            is_passwd_change.set(false)
        }
        (false, false) if passwd() == confirm_passwd() => {
            update_status.set("Passwords matched!".to_string());
            is_passwd_change.set(true);
        }
        _ => update_status.set("Passords do not match!".to_string()),
    };
    rsx! {
        div { class: "fixed inset-0 z-50 flex items-center justify-center bg-gray-900 bg-opacity-60",
            document::Title { "Settings" }
            div { class: "block rounded-lg bg-white dark:bg-gray-800 w-2/5 p-4 shadow-[0_2px_15px_-3px_rgba(0,0,0,0.07),0_10px_20px_-2px_rgba(0,0,0,0.04)] z-70",
                h5 { class: "mb-5 text-xl font-medium leading-tight text-neutral-800 dark:text-gray-200",
                    "Update Your Settings."
                }
                form {
                    div { class: "mb-5",
                        input {
                            class: "input-field-common",
                            name: "image",
                            r#type: "text",
                            placeholder: "URL of profile picture",
                            oninput: move |evt| {
                                image_url.set(evt.value());
                                no_image_url_yet.set(false);
                            },
                            value: if no_image_url_yet() { user_settings().image() } else { Some(image_url()) },
                        }
                    }
                    div { class: "mb-5",
                        input {
                            class: "input-field-common",
                            name: "username",
                            disabled: true,
                            r#type: "text",
                            placeholder: user_settings().username(),
                        }
                    }
                    div { class: "mb-5",
                        textarea {
                            name: "bio",
                            class: "input-field-common",
                            placeholder: "Short bio about you",
                            oninput: move |evt| {
                                bio.set(evt.value());
                                no_bio_yet.set(false);
                            },
                            value: if no_bio_yet() { user_settings().bio() } else { Some(bio()) },
                        }
                    }
                    div { class: "mb-5",
                        input {
                            class: "input-field-common",
                            name: "email",
                            r#type: "text",
                            placeholder: "Email (required)",
                            required: true,
                            oninput: move |evt| {
                                email.set(evt.value());
                                no_email_yet.set(false);
                            },
                            value: if no_email_yet() { user_settings().email() } else { email() },
                        }
                    }
                    div { class: "mb-5 relative",
                        input {
                            name: "password",
                            class: "input-field-common",
                            placeholder: "New password",
                            r#type: format!("{}", if passwd_visible() { "text" } else { "password" }),
                            oninput: move |evt| {
                                passwd.set(evt.value());
                                passwords_match()
                            },
                        }
                        span {
                            class: "absolute inset-y-0 right-0 flex items-center pr-3 cursor-pointer text-gray-500 dark:text-gray-400",
                            onclick: move |_| passwd_visible.toggle(),
                            i { class: format!("{}", if passwd_visible() { "far fa-eye" } else { "far fa-eye-slash" }) }
                        }
                    }
                    div { class: "mb-5 relative",
                        input {
                            name: "confirm_password",
                            class: "input-field-common",
                            placeholder: "Confirm password",
                            r#type: format!("{}", if passwd_visible() { "text" } else { "password" }),
                            oninput: move |evt| {
                                confirm_passwd.set(evt.value());
                                passwords_match()
                            },
                        }
                        span {
                            class: "absolute inset-y-0 right-0 flex items-center pr-3 cursor-pointer  text-gray-500",
                            onclick: move |_| passwd_visible.toggle(),
                            i { class: format!("{}", if passwd_visible() { "far fa-eye" } else { "far fa-eye-slash" }) }
                        }
                    }
                    div { class: "mb-5",
                        p {
                            class: format!(
                                "font-medium {}",
                                if update_status().starts_with("Successful.")
                                    || update_status().starts_with("Passwords matched")
                                {
                                    "text-green-500"
                                } else {
                                    "text-red-500"
                                },
                            ),
                            {update_status()}
                        }
                    }
                    div { class: "flex justify-between mb-5",
                        input {
                            class: "bg-blue-700 hover:bg-blue-800 px-5 py-3 text-white rounded-lg disabled:bg-gray-400 disabled:cursor-not-allowed focus:outline-none focus:ring-2 focus:ring-blue-700",
                            r#type: "button",
                            disabled: match (
                                no_bio_yet(),
                                no_email_yet(),
                                no_image_url_yet(),
                                ((user_settings().bio().is_none() && bio().is_empty())
                                    || user_settings().bio() == Some(bio())),
                                (user_settings().email() == email()),
                                ((user_settings().image().is_none() && image_url().is_empty())
                                    || user_settings().image() == Some(image_url())),
                                passwd().is_empty(),
                                confirm_passwd().is_empty(),
                            ) {
                                (true, true, true, _, _, _, true, true) => true,
                                (false, false, false, true, true, true, true, true) => true,
                                (true, false, false, _, true, true, _, _) => true,
                                (false, true, false, true, _, true, _, _) => true,
                                (false, false, true, true, true, _, _, _) => true,
                                (false, true, true, true, _, _, _, _) => true,
                                (true, false, true, _, true, _, _, _) => true,
                                (true, true, false, _, _, true, _, _) => true,
                                (_, _, _, _, _, _, false, false) if passwd() == confirm_passwd() => false,
                                _ => false,
                            },
                            onclick: on_update,
                            value: {
                                format!(
                                    "{}",
                                    if passwd() == confirm_passwd() && !passwd().is_empty()
                                        && !confirm_passwd().is_empty()
                                    {
                                        "Reset Password & Logout"
                                    } else {
                                        "Update Settings"
                                    },
                                )
                            },
                        }
                        button {
                            r#type: "button",
                            class: format!(
                                "{}",
                                if update_status().starts_with("Successful.") {
                                    "btn-primary"
                                } else {
                                    "bg-gray-300 hover:bg-gray-400 dark:bg-gray-600 dark:hover:bg-gray-500 px-5 py-3 text-white rounded-lg"
                                },
                            ),
                            onclick: on_cancel,
                            {
                                format!(
                                    "{}",
                                    if update_status().starts_with("Successful.") {
                                        "Back to Home"
                                    } else {
                                        "Cancel"
                                    },
                                )
                            }
                        }
                    }
                }
            }
        }
    }
}
