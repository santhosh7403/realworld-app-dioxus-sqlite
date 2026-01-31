use dioxus::prelude::*;

use dioxus::router::root_router;
use views::{Home, Profile};
use views::{Login, ResetPasswd, Settings, SignUp};

use crate::models::{Pagination, User};
use crate::views::Article;
use crate::views::Editor;

mod views;

mod auth;
mod components;
mod database;
mod models;

#[derive(Routable, Clone)]
#[rustfmt::skip]
enum Route {
    #[layout(NavBar)]
        #[route("/")]
        Home {},
        #[route("/profile/:profile_user")]
        Profile{profile_user: String},
        #[end_layout]
        #[route("/newarticle")]
        NewArticle {},
        #[route("/article/:slug")]
        Article {slug: String },
        #[route("/editor/:slug")]
        Editor{slug: String},
        #[route("/login")]
        Login {},
        #[route("/signup")]
        SignUp{},
        #[route("/settings")]
        Settings{},
        #[route("/:..route")]
        PageNotFound {
            route: Vec<String>,
        },
        #[route("/reset_password?:token")]
        ResetPasswd{
            token: String
        },

}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    #[cfg(feature = "web")]
    {
        dioxus::launch(App);
        // use tracing::Level;
        // dioxus_logger::init(Level::INFO).expect("failed to init logger");
    }
    #[cfg(feature = "server")]
    {
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async move {
                use tracing::Level;

                dioxus_logger::init(Level::INFO).expect("failed to init logger");
                launch_server(App).await;
            });
    }
}

#[cfg(feature = "server")]
async fn launch_server(_component: fn() -> Element) {
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    database::server::init_db()
        .await
        .expect("Problem during initialization of the database");

    let ip =
        dioxus_cli_config::server_ip().unwrap_or_else(|| IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
    let port = dioxus_cli_config::server_port().unwrap_or(8080);
    let address = SocketAddr::new(ip, port);
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    let router = axum::Router::new()
        .serve_dioxus_application(ServeConfig::new(), App)
        // .serve_dioxus_application(ServeConfig::new().unwrap(), App)
        .layer(axum::middleware::from_fn(crate::auth::auth_middleware))
        .into_make_service();

    axum::serve(listener, router).await.unwrap();
}

#[derive(Clone, Default)]
struct LoggedInUser(Option<User>);

#[derive(Clone, Default)]
struct SearchMeta {
    page: i64,
    amount: i64,
}

#[derive(Clone, Default)]
struct SearchWindow(bool);

#[derive(Clone, Default)]
struct SearchString(String);

#[derive(Clone, Default)]
struct PageAmount(i64);

#[derive(Clone, Default)]
struct ThemeMode(String);

#[component]
fn App() -> Element {
    // Build cool things ✌️
    use_context_provider(|| Signal::new(LoggedInUser(None)));
    use_context_provider(|| Signal::new(models::Pagination::default()));
    use_context_provider(|| {
        Signal::new(SearchMeta {
            page: 0,
            amount: 10,
        })
    });
    use_context_provider(|| Signal::new(SearchWindow(false)));
    use_context_provider(|| Signal::new(SearchString(String::new())));
    use_context_provider(|| Signal::new(PageAmount(10)));
    use_context_provider(|| Signal::new(ThemeMode(String::from("dark"))));

    use_effect(move || {
        let mut search_meta = use_context::<Signal<SearchMeta>>();
        let page_amount = use_context::<Signal<PageAmount>>();

        search_meta.set(SearchMeta {
            page: 0,
            amount: page_amount().0,
        });
    });

    use_effect(move || {
        let logged_user = use_context::<Signal<LoggedInUser>>();
        let mut page_amount = use_context::<Signal<crate::PageAmount>>();
        if let Some(user) = logged_user().0 {
            page_amount.set(crate::PageAmount(user.per_page_amount()));
        }
        #[cfg(feature = "web")]
        {
            if let Some(window) = web_sys::window() {
                if let Ok(Some(storage)) = window.local_storage() {
                    if let Ok(Some(saved_theme)) = storage.get_item("theme") {
                        let mut theme = use_context::<Signal<ThemeMode>>();
                        theme.set(ThemeMode(saved_theme.clone()));

                        if let Some(document) = window.document() {
                            if let Some(html) = document.document_element() {
                                if saved_theme == "dark" {
                                    let _ = html.class_list().add_1("dark");
                                } else {
                                    let _ = html.class_list().remove_1("dark");
                                }
                            }
                        }
                    }
                }
            }
        }
    });
    rsx! {
        // Global app resources
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        document::Link {
            rel: "stylesheet",
            href: "https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.5.1/css/all.min.css",
            integrity: "sha512-DTOQO9RWCH3ppGqcWaEA1BIZOC6xxalwEsw9c2QQeAIftl+Vegovlnee1c9QX4TctnWMn13TZye+giMm8e2LwA==",
            crossorigin: "anonymous",
            referrerpolicy: "no-referrer",
        }
        document::Title { "My App" }
        div { Router::<Route> {} }
    }
}

#[component]
fn Footer() -> Element {
    rsx! {
        div { class: "bg-gray-200 text-gray-600 dark:bg-gray-900 dark:text-gray-400 text-center text-xs sticky bottom-0",
            a { href: "/", "MyApp" }
            div { class: "text-gray-600 dark:text-gray-400", "© 2026 My-Website" }
        }
    }
}

#[component]
fn NavBar() -> Element {
    let mut logged_user = use_context::<Signal<LoggedInUser>>();
    let nav = navigator();

    let on_submit = move |evt: FormEvent| async move {
        evt.prevent_default();
        match auth::logout().await {
            Ok(_) => {
                logged_user.set(LoggedInUser(None));
                nav.replace(crate::Route::Home {});
            }
            Err(_) => (),
        }
    };

    let on_click_home = move |_| {
        let mut pagination = use_context::<Signal<Pagination>>();
        let page_amount = use_context::<Signal<crate::PageAmount>>();
        let mut search_window = use_context::<Signal<SearchWindow>>();
        let pagination_string = pagination()
            .set_amount(page_amount().0)
            .set_my_feed(false)
            .reset_page()
            .to_string();
        nav.push(pagination_string);
        search_window.set(SearchWindow(false));

        let router_context = root_router();
        let mut route_string = String::new();
        if let Some(context) = router_context {
            route_string = context.full_route_string()
        }
        if pagination() != Pagination::from(route_string.clone()) {
            pagination.set(Pagination::from(route_string.clone()));
        }
    };

    rsx! {
        nav { class: "sticky top-0 z-10 shadow-md",
            div { class: "bg-gray-800 text-white shadow-lg md:relative md:top-0 md:left-0 md:right-auto md:w-full rounded-b-xl px-4 py-3 md:py-4",

                div { class: "flex justify-around items-center",
                    button { onclick: on_click_home, r#type: "button",
                        div { class: "group navitem",
                            i { class: "fas fa-home navitem-icon" }
                            span { class: "text-xs md:text-base mt-1 font-semibold",
                                "Home"
                            }
                        }
                    }


                    if logged_user().0.is_some() {
                        Link { to: Route::NewArticle {},
                            div { class: "group navitem",
                                i { class: "fa-solid fa-square-plus navitem-icon" }
                                span { class: "text-xs md:text-base mt-1 font-semibold",
                                    "New Article"
                                }
                            }
                        }
                        Link { to: Route::Settings {},
                            div { class: "group navitem",
                                i { class: "fa-solid fa-gear navitem-icon" }
                                span { class: "text-xs md:text-base mt-1 font-semibold",
                                    "Settings"
                                }
                            }
                        }
                        Link {
                            to: Route::Profile {
                                profile_user: (logged_user().0.unwrap().username),
                            },
                            div { class: "group navitem",
                                i { class: "fa-regular fa-circle-user navitem-icon" }
                                span { class: "text-xs md:text-base mt-1 font-semibold",
                                    // "user1"
                                    {logged_user().0.unwrap().username}
                                                                // {current_user().0}
                                }
                            }
                        }
                        form { onsubmit: on_submit,
                            button {
                                div { class: "group navitem",
                                    i { class: "fa-solid fa-right-from-bracket navitem-icon" }
                                    span { class: "text-xs md:text-base mt-1 font-semibold",
                                        "Logout"
                                    }
                                }
                            }
                        }
                    } else {
                        Link { to: Route::SignUp {},
                            div { class: "group navitem",
                                i { class: "fa-solid fa-user-plus navitem-icon" }
                                span { class: "text-xs md:text-base mt-1 font-semibold",
                                    "Sign up"
                                }
                            }
                        }


                        Link { to: Route::Login {},
                            div { class: "group navitem",
                                i { class: "fa-solid fa-right-to-bracket  navitem-icon" }
                                span { class: "text-xs md:text-base mt-1 font-semibold",
                                    "Login"
                                }
                            }
                        }
                    }

                    // Theme Toggle
                    button {
                        onclick: move |_| {
                            let mut theme = use_context::<Signal<ThemeMode>>();
                            let new_theme = if theme().0 == "dark" { "light" } else { "dark" };
                            theme.set(ThemeMode(new_theme.to_string()));

                            #[cfg(feature = "web")]
                            {
                                if let Some(window) = web_sys::window() {
                                    if let Ok(Some(storage)) = window.local_storage() {
                                        let _ = storage.set_item("theme", new_theme);
                                    }
                                    if let Some(document) = window.document() {
                                        if let Some(html) = document.document_element() {
                                            if new_theme == "dark" {
                                                let _ = html.class_list().add_1("dark");
                                            } else {
                                                let _ = html.class_list().remove_1("dark");
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        r#type: "button",
                        class: "border-l border-gray-600 pl-3 ml-2",
                        div { class: "group navitem",
                            if use_context::<Signal<ThemeMode>>().read().0 == "dark" {
                                // Sun icon for dark mode
                                svg {
                                    class: "w-6 h-6 md:w-7 md:h-7 text-yellow-400 transition-transform group-hover:scale-110",
                                    fill: "none",
                                    view_box: "0 0 24 24",
                                    stroke_width: "1.5",
                                    stroke: "currentColor",
                                    path {
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        d: "M12 3v2.25m6.364.386l-1.591 1.591M21 12h-2.25m-.386 6.364l-1.591-1.591M12 18.75V21m-4.773-4.227l-1.591 1.591M5.25 12H3m4.227-4.773L5.636 5.636M15.75 12a3.75 3.75 0 11-7.5 0 3.75 3.75 0 017.5 0z"
                                    }
                                }
                            } else {
                                // Moon icon for light mode
                                svg {
                                    class: "w-6 h-6 md:w-7 md:h-7 text-gray-400 transition-transform group-hover:scale-110",
                                    fill: "none",
                                    view_box: "0 0 24 24",
                                    stroke_width: "1.5",
                                    stroke: "currentColor",
                                    path {
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        d: "M21.752 15.002A9.718 9.718 0 0118 15.75c-5.385 0-9.75-4.365-9.75-9.75 0-1.33.266-2.597.748-3.752A9.753 9.753 0 003 11.25C3 16.635 7.365 21 12.75 21a9.753 9.753 0 009.002-5.998z"
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Outlet::<Route> {}
        }
    }
}

#[component]
fn NewArticle() -> Element {
    let slug = String::new();
    rsx! {
        Editor { slug }

        Outlet::<Route> {}
    }
}

#[component]
fn PageNotFound(route: Vec<String>) -> Element {
    rsx! {
        h1 { "Page not found" }
        p { "We are terribly sorry, but the page you requested doesn't exist." }
        pre { color: "red", "log:\nattemped to navigate to: {route:?}" }
    }
}
