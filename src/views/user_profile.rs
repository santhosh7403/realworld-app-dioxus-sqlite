use crate::{
    components::{ArticlePreviewList, ItemsPerPage, PrevNextButton},
    models::Pagination,
};
use dioxus::{document, prelude::*};

#[server]
#[tracing::instrument]
pub async fn profile_articles(
    username: String,
    favourites: bool,
    page: i64,
    amount: i64,
) -> Result<Vec<crate::models::Article>, ServerFnError> {
    let page = i64::from(page);
    let amount = i64::from(amount);
    let server_context = server_context();
    let request_parts: axum::http::request::Parts = server_context.extract().await?;

    crate::models::Article::for_user_profile_home(
        username, // favourites.unwrap_or_default(),
        favourites,
        page,
        amount,
        request_parts,
    )
    .await
    .map_err(|x| {
        let err = format!("Error while getting user_profile articles: {x:?}");
        tracing::error!("{err}");
        ServerFnError::ServerError("Could not retrieve articles, try again later".into())
    })
}

#[component]
pub fn Profile(profile_user: ReadOnlySignal<String>) -> Element {
    let mut logged_user = use_context::<Signal<crate::LoggedInUser>>();

    let _ = use_resource(move || async move {
        match crate::auth::current_user().await {
            Ok(res_user) => {
                logged_user.set(crate::LoggedInUser(Some(res_user)));
            }
            Err(_) => (),
        }
    });

    rsx! {
        div { class: "bg-gray-200",
            div { class: "mx-auto sm:px-6 lg:px-8 bg-gray-200 px-2 py-2 sm:px-0 text-gray-800",
                ProfilePage {
                    profile_user: profile_user(),
                    route_path: format!("/profile/{}", profile_user()),
                }
            }
        }
    }
}

#[component]
pub fn ProfilePage(
    profile_user: ReadOnlySignal<String>,
    route_path: ReadOnlySignal<String>,
) -> Element {
    let pagination = use_context::<Signal<Pagination>>();
    let logged_user = use_context::<Signal<crate::LoggedInUser>>();

    let articles_resource = use_resource(move || async move {
        profile_articles(
            profile_user(),
            pagination().get_favourites(),
            pagination().get_page(),
            pagination().get_amount(),
        )
        .await
    });

    rsx! {
        match &*articles_resource.read() {
            Some(Ok(articles)) => {
                rsx! {
                    div { class: "mb-5",
                        div { class: "flex justify-between px-2 bg-gray-200 py-2",
                            div { class: "flex text-gray-800",
                                UserArticlesTab { user: profile_user() }
                                FavouritedArticlesTab { user: profile_user() }
                            }
                            ItemsPerPage { route_path: route_path() }
                        }
                        UserInfo { user: profile_user() }
                        ArticlePreviewList { articles: articles.clone(), logged_user: logged_user().0 }
                        div { class: "flex justify-between",
                            div { class: "flex gap-4 mb-4",
                                PrevNextButton { articles: articles.clone(), route_path: route_path() }
                            }
                            div { class: "flex justify-end px-7", BackToHome {} }
                        }
                        crate::Footer {}
                    }
                }
            }
            Some(Err(e)) => rsx! {
                div { class: "text-gray-800", "Failed to load: {e}" }
            },
            None => rsx! {
                div { class: "text-gray-800", "Loading Articles..." }
            },
        }
    }
}

#[component]
fn BackToHome() -> Element {
    let on_click = move |_| {
        let nav = navigator();
        let mut pagination = use_context::<Signal<Pagination>>();
        let page_amount = use_context::<Signal<crate::PageAmount>>();
        let pagination_string = pagination()
            .set_amount(page_amount().0)
            .set_my_feed(false)
            .reset_page()
            .to_string();
        nav.push(pagination_string);
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
        h4 {
            onclick: on_click,
            class: "text-blue-500 underline cursor-pointer",
            "Back to Home"
        }
    }
}

#[component]
fn UserArticlesTab(user: ReadOnlySignal<String>) -> Element {
    let mut pagination = use_context::<Signal<Pagination>>();
    let page_amount = use_context::<Signal<crate::PageAmount>>();

    let nav = navigator();
    let on_click = move |_| {
        nav.push(format!(
            "/profile/{}{}",
            user(),
            pagination()
                .reset_page()
                .set_favourites(false)
                .set_amount(page_amount().0)
                .to_string()
        ));
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
        div { class: "mb-5 px-2",
            document::Title { "Profile of {user()}" }
            button {
                onclick: on_click,
                r#type: "button",
                class: format!(
                    "font-bold {}",
                    if !pagination().get_favourites() { "border-b-8" } else { "cursor-pointer" },
                ),

                {user()}
                "'s Articles"
            }
        }
    }
}

#[component]
fn FavouritedArticlesTab(user: ReadOnlySignal<String>) -> Element {
    let mut pagination = use_context::<Signal<Pagination>>();
    let page_amount = use_context::<Signal<crate::PageAmount>>();

    let nav = navigator();
    let on_click = move |_| {
        nav.push(format!(
            "/profile/{}{}",
            user(),
            pagination()
                .set_favourites(true)
                .reset_page()
                .set_amount(page_amount().0)
                .to_string()
        ));
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
        div { class: "mb-5 px-2",
            button {
                onclick: on_click,
                r#type: "button",
                class: format!(
                    "font-bold {}",
                    if pagination().get_favourites() { "border-b-8" } else { "cursor-pointer" },
                ),
                "Favourited Articles"
            }
        }
    }
}

#[component]
fn UserInfo(user: ReadOnlySignal<String>) -> Element {
    let user_resource = use_resource(move || async move { user_profile(user()).await });
    rsx! {

        match &*user_resource.read() {
            Some(Ok(profile_model)) => {
                let image = profile_model.user.image();
                let username = profile_model.user.username();
                let bio = profile_model.user.bio();
                let email = format!(
                    "{}",
                    if profile_model.user.email().is_empty() {
                        " - ".to_string()
                    } else {
                        profile_model.user.email()
                    },
                );
                rsx! {
                    div { class: "bg-white text-gray-800 mb-2 p-4",
                        div { class: "mb-5 px-5 flex justify-between",
                            h2 { class: "font-bold text-xl underline",
                                "Profile data of User - "
                                {username.clone()}
                            }
                            BackToHome {}
                        }
                        div { class: "flex",
                            div { class: "mb-4",
                                img { src: image, class: "w-10 h-10 rounded-full" }
                            }
                            div { class: "px-4",
                                h4 { {username} }
                            }
                        }
                        p { class: "",
                            "Bio: "
                            {bio.unwrap_or("No bio available".into())}
                        }
                        div { class: "",
                            "Email: "
                            {email}
                        }
                    }
                }
            }
            Some(Err(e)) => rsx! {
                div { class: "text-gray-800", "Failed to load: {e}" }
            },
            None => rsx! {
                div { class: "text-gray-800", "Loading Articles..." }
            },
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct UserProfileModel {
    user: crate::models::User,
    following: Option<bool>,
}

#[server]
#[tracing::instrument]
pub async fn user_profile(username: String) -> Result<UserProfileModel, ServerFnError> {
    let user = crate::models::User::get(username.clone())
        .await
        .map_err(|x| {
            let err = format!("Error while getting user in user_profile: {x:?}");
            tracing::error!("{err}");
            ServerFnError::new("Could not retrieve articles, try again later")
        })?;
    let mut following = None;
    let server_context = server_context();
    let request_parts: axum::http::request::Parts = server_context.extract().await?;

    if let Some(logged_user) = crate::auth::get_username(request_parts) {
        if sqlx::query_scalar!(
            "
            Select count(*) from Follows where follower=$2 and influencer=$1
            ",
            username,
            logged_user
        )
        .fetch_one(crate::database::server::get_db())
        .await?
            == 1
        {
            following = Some(true);
        }
    }
    Ok(UserProfileModel { user, following })
}
