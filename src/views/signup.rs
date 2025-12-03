use crate::auth::SignupResponse;
use dioxus::prelude::*;

#[component]
pub fn SignUp() -> Element {
    let mut passwd_visible = use_signal(|| false);
    let mut signup_status = use_signal(|| String::new());
    let mut create_button_string = use_signal(|| String::from("Create Account"));
    let mut username = use_signal(|| String::new());
    let mut email = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());

    let on_cancel = move |_| {
        let nav = navigator();
        if nav.can_go_back() {
            nav.go_back();
        } else {
            nav.replace(crate::Route::Home {});
        }
    };

    let on_click = move |_| async move {
        if signup_status().starts_with("Signup Successful") {
            let nav = navigator();
            nav.replace(crate::Route::Login {});
        } else {
            let res_signup = crate::auth::signup_action(username(), email(), password()).await;

            match res_signup {
                Ok(SignupResponse::Success) => {
                    signup_status.set("Signup Successful".to_string());
                    create_button_string.set("Login to Account Created?".to_string());
                }
                Ok(SignupResponse::ValidationError(validation_error)) => {
                    signup_status.set(format!("Problem while validating: {validation_error}."))
                }

                Ok(SignupResponse::CreateUserError(create_error)) => {
                    signup_status.set(format!("Problem while creating user:s {create_error}."))
                }

                Err(err) => {
                    tracing::error!("Problem during signup: {err:?}");
                    signup_status
                        .set("There was some problem with signup, try again later".to_string());
                }
            }
        }
    };

    rsx! {
        div { class: "fixed inset-0 z-50 flex items-center justify-center bg-gray-900 bg-opacity-60",
            div { class: "block rounded-lg bg-white w-2/5 p-4 shadow-[0_2px_15px_-3px_rgba(0,0,0,0.07),0_10px_20px_-2px_rgba(0,0,0,0.04)] z-70",

                h5 { class: "mb-5 text-xl font-medium leading-tight text-neutral-800",
                    "Create an account."
                }
                document::Title { "Account Creation" }
                p { class: "text-xs-center py-6",
                    span { class: "text-blue-500 font-medium",
                        a { href: "/login", "Have an account already? Click here to login " }
                    }
                }

                form {
                    label {
                        class: "block text-gray-700 text-sm font-bold mb-2",
                        r#for: "username",
                        "User Name"
                    }

                    div { class: "mb-5",
                        input {
                            class: "shadow appearance-none rounded w-full py-2 px-3 text-gray-700 leading-tight focus:ring",
                            id: "username",
                            name: "username",
                            r#type: "text",
                            placeholder: "username",
                            required: true,
                            oninput: move |evt| username.set(evt.value()),
                        }
                    }

                    label {
                        class: "block text-gray-700 text-sm font-bold mb-2",
                        r#for: "email",
                        "Email"
                    }

                    div { class: "mb-5",
                        input {
                            class: "shadow appearance-none rounded w-full py-2 px-3 text-gray-700 leading-tight focus:ring",
                            id: "email",
                            name: "email",
                            r#type: "email",
                            placeholder: "Email",
                            required: true,
                            oninput: move |evt| email.set(evt.value()),
                        }
                    }
                    label {
                        class: "block text-gray-700 text-sm font-bold mb-2",
                        r#for: "password",
                        "Password"
                    }

                    div { class: "mb-5 relative",
                        input {
                            class: "shadow appearance-none rounded w-full py-2 px-3 text-gray-700 leading-tight focus:ring",
                            id: "password",
                            name: "password",
                            r#type: format!("{}", if passwd_visible() { "text" } else { "password" }),
                            placeholder: "Password",
                            required: true,
                            oninput: move |evt| password.set(evt.value()),
                        }
                        span {
                            class: "absolute inset-y-0 right-0 flex items-center pr-3 cursor-pointer text-gray-500",
                            onclick: move |_| passwd_visible.toggle(),
                            i { class: format!("{}", if passwd_visible() { "far fa-eye" } else { "far fa-eye-slash" }) }
                        }
                    }
                    div { class: "flex flex-row-reverse space-x-4 space-x-reverse",
                        button {
                            r#type: "button",
                            onclick: on_click,
                            class: "bg-blue-700 hover:bg-blue-800 px-5 py-3 text-white rounded-lg",

                            {create_button_string()}
                        }
                        button {
                            r#type: "button",
                            class: "bg-gray-300 hover:bg-gray-400 px-5 py-3 text-white rounded-lg",
                            onclick: on_cancel,
                            "Cancel"
                        }
                    }

                    div {
                        span {
                            class: format!(
                                "font-medium {}",
                                if signup_status().starts_with("Signup Successful") {
                                    "text-green-500"
                                } else {
                                    "text-red-500"
                                },
                            ),
                            {signup_status()}
                        }
                    }
                }
            }
        }
    }
}
