use crate::auth;
use dioxus::prelude::*;

#[component]
pub fn Login() -> Element {
    let nav = navigator();
    let mut passwd_visible = use_signal(|| false);
    let mut login_status = use_signal(|| String::new());

    let on_submit = move |evt: FormEvent| async move {
        evt.prevent_default();
        let username_data = evt
            .values()
            .into_iter()
            .filter(|d| d.0 == "username")
            .last();
        let username = match username_data {
            Some((_, FormValue::Text(value))) => value,
            _ => String::new(),
        };

        let password_data = evt
            .values()
            .into_iter()
            .filter(|d| d.0 == "password")
            .last();
        let password = match password_data {
            Some((_, FormValue::Text(value))) => value,
            _ => String::new(),
        };
        let login_result = auth::login(username.clone(), password).await;

        match login_result {
            Ok(_) => match auth::current_user().await {
                Ok(Some(user)) if user.username() == username => {
                    nav.replace(crate::Route::Home {});
                }
                _ => login_status.set("Login failed! : username or password incorrect".to_string()),
            },
            Err(err) => {
                tracing::debug!("login failed! : {err}");
                login_status.set("Login failed! : username or password incorrect".to_string())
            }
        }
    };

    rsx! {
        div { class: "fixed inset-0 z-50 flex items-center justify-center bg-gray-900 bg-opacity-60",
            div { class: "block rounded-lg bg-white dark:bg-gray-800 w-2/5 p-4 shadow-[0_2px_15px_-3px_rgba(0,0,0,0.07),0_10px_20px_-2px_rgba(0,0,0,0.04)] z-70",
                h5 { class: "mb-5 text-xl font-medium leading-tight text-gray-800 dark:text-gray-200",
                    " Please login with your credentials."
                }
                document::Title { "Login to MyApp" }
                form { class: "mb-10", id: "login", onsubmit: on_submit,

                    label {
                        class: "block text-gray-700 dark:text-gray-300 text-sm font-bold",
                        r#for: "username",
                        "User Name"
                    }

                    div { class: "mb-5",
                        input {
                            class: "shadow appearance-none border dark:border-gray-600 rounded w-full py-2 px-3 text-gray-700 dark:text-gray-200 dark:bg-gray-700 leading-tight focus:outline-none focus:shadow-outline",
                            id: "username",
                            name: "username",
                            r#type: "text",
                            placeholder: "user name",
                        }
                    }

                    label {
                        class: "block text-gray-700 dark:text-gray-300 text-sm font-bold",
                        r#for: "password",
                        "Password"
                    }

                    div { class: "mb-5 relative",
                        input {
                            class: "shadow appearance-none border dark:border-gray-600 rounded w-full py-2 px-3 text-gray-700 dark:text-gray-200 dark:bg-gray-700 leading-tight focus:outline-none focus:shadow-outline",
                            id: "password",
                            name: "password",
                            r#type: format!("{}", if passwd_visible() { "text" } else { "password" }),
                            placeholder: "password",
                        }
                        span {
                            class: "absolute inset-y-0 right-0 flex items-center pr-3 cursor-pointer text-gray-500 dark:text-gray-400",
                            onclick: move |_| passwd_visible.toggle(),
                            i { class: format!("{}", if passwd_visible() { "far fa-eye" } else { "far fa-eye-slash" }) }
                        }
                    }

                    div { class: "flex flex-row-reverse space-x-4 space-x-reverse",

                        button { class: "bg-blue-700 hover:bg-blue-800 px-5 py-2 text-white rounded-lg",
                            "Signin"
                        }
                        button {
                            class: "bg-gray-300 hover:bg-gray-400 dark:bg-gray-600 dark:hover:bg-gray-500 px-5 py-2 text-white rounded-lg",
                            r#type: "button",
                            onclick: move |_| {
                                if nav.can_go_back() {
                                    nav.go_back();
                                } else {
                                    nav.replace(crate::Route::Home {});
                                }
                            },
                            "Cancel"
                        }
                        a {
                            class: "p-2 text-gray-700 dark:text-gray-300 hover:text-blue-500 hover:underline",
                            href: "/reset_password",
                            "Forgot password?"
                        }
                    }
                    div {
                        p { class: "block text-red-700 text-sm font-bold", {login_status()} }
                    }
                }
            }
        }
    }
}
