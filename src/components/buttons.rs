use crate::models::Pagination;
use dioxus::prelude::*;

#[server]
#[tracing::instrument]
pub async fn fav_action(slug: String) -> Result<bool, ServerFnError> {
    let server_context = server_context();
    let request_parts: axum::http::request::Parts = server_context.extract().await?;
    let Some(username) = crate::auth::get_username(request_parts) else {
        return Err(ServerFnError::ServerError(
            "You need to be authenticated".into(),
        ));
    };
    toggle_fav(slug, username).await.map_err(|x| {
        tracing::error!("problem while updating the database: {x:?}");
        ServerFnError::ServerError("error while updating the follow".into())
    })
}

#[cfg(feature = "server")]
#[tracing::instrument]
async fn toggle_fav(slug: String, username: String) -> Result<bool, sqlx::Error> {
    let db = crate::database::server::get_db();
    match sqlx::query!(
        "SELECT * FROM FavArticles WHERE article=$1 and username=$2",
        slug,
        username
    )
    .fetch_one(db)
    .await
    {
        Ok(_) => sqlx::query!(
            "DELETE FROM FavArticles WHERE article=$1 and username=$2",
            slug,
            username
        )
        .execute(db)
        .await
        .map(|_| false),
        Err(sqlx::error::Error::RowNotFound) => sqlx::query!(
            "INSERT INTO FavArticles(article, username) VALUES ($1, $2)",
            slug,
            username
        )
        .execute(db)
        .await
        .map(|_| true),
        Err(x) => Err(x),
    }
}

#[component]
pub fn ButtonFav(article_detail: ReadOnlySignal<crate::views::ArticleDetailed>) -> Element {
    let mut fav = use_signal(|| article_detail().article.fav);
    let mut fav_count = use_signal(|| article_detail().article.favorites_count);
    let pagination = use_context::<Signal<Pagination>>();

    let on_submit = move |evt: FormEvent| async move {
        let res = fav_action(evt.values()["slug"].as_value()).await;
        match res {
            Ok(_) => {
                fav.set(!fav());
                if fav() {
                    fav_count.set(fav_count() + 1)
                } else {
                    fav_count.set(fav_count() - 1)
                }
            }
            Err(_) => (),
        }
    };

    rsx! {
        if article_detail().logged_user.is_some() && !pagination().get_favourites() {
            if article_detail().logged_user.unwrap().username
                == article_detail().article.author.username
            {
                div { class: "flex items-center gap-2",
                    button { class: "text-gray-600 ",
                        i { class: "far fa-star" }
                        span { class: "cursor-not-allowed", " My Favourite" }
                    }
                }
            } else {
                // if fav() {
                div { class: "flex items-center gap-2",
                    form { onsubmit: on_submit,
                        input {
                            r#type: "hidden",
                            name: "slug",
                            value: article_detail().article.slug,
                        }
                        button {
                            class: format!(
                                "{}",
                                if fav() {
                                    "text-yellow-500 hover:text-gray-500 transition-colors duration-200"
                                } else {
                                    "text-gray-500 hover:text-yellow-500 transition-colors duration-200"
                                },
                            ),
                            i { class: format!("{}", if fav() { "fas fa-star" } else { "far fa-star" }) }
                            span { " My Favourite" }
                        }
                    }
                }
            }
        }
        div { class: format!("{}", if fav_count() > 0 { "px-8 text-yellow-500" } else { "px-8" }),
            span {
                " Favourites: "
                {fav_count().to_string()}
            }
        }
    }
}

#[component]
pub fn ButtonFavFavourited(
    article_detail: ReadOnlySignal<crate::views::ArticleDetailed>,
) -> Element {
    rsx! {

        div {
            class: format!(
                "{}",
                if article_detail().article.favorites_count > 0 {
                    "px-8 text-yellow-500"
                } else {
                    "px-8"
                },
            ),
            span {
                " Favourites: "
                {article_detail().article.favorites_count.to_string()}
            }
        }
    }
}

#[server]
#[tracing::instrument]
pub async fn follow_action(other_user: String) -> Result<bool, ServerFnError> {
    let server_context = server_context();
    let request_parts: axum::http::request::Parts = server_context.extract().await?;
    let Some(username) = crate::auth::get_username(request_parts) else {
        return Err(ServerFnError::ServerError(
            "You need to be authenticated".into(),
        ));
    };
    toggle_follow(username, other_user).await.map_err(|x| {
        tracing::error!("problem while updating the database: {x:?}");
        ServerFnError::ServerError("error while updating the follow".into())
    })
}

#[cfg(feature = "server")]
#[tracing::instrument]
async fn toggle_follow(current: String, other: String) -> Result<bool, sqlx::Error> {
    let db = crate::database::server::get_db();
    match sqlx::query!(
        "SELECT * FROM Follows WHERE follower=$1 and influencer=$2",
        current,
        other
    )
    .fetch_one(db)
    .await
    {
        Ok(_) => sqlx::query!(
            "DELETE FROM Follows WHERE follower=$1 and influencer=$2",
            current,
            other
        )
        .execute(db)
        .await
        .map(|_| false),
        Err(sqlx::error::Error::RowNotFound) => sqlx::query!(
            "INSERT INTO Follows(follower, influencer) VALUES ($1, $2)",
            current,
            other
        )
        .execute(db)
        .await
        .map(|_| true),
        Err(x) => Err(x),
    }
}

#[component]
pub fn ButtonFollow(article_detail: ReadOnlySignal<crate::views::ArticleDetailed>) -> Element {
    let mut button_disable = use_signal(|| false);
    let mut mouse_hover = use_signal(|| false);
    let mut button_icon = use_signal(|| String::new());
    let mut button_text = use_signal(|| String::new());
    let mut button_class = use_signal(|| String::new());
    let mut is_following = use_signal(|| article_detail().article.author.following);
    let on_submit = move |_| async move {
        button_disable.set(true);
        let res = follow_action(article_detail().article.author.username).await;
        match res {
            Ok(_) => {
                button_disable.set(false);
                is_following.toggle();
            }
            Err(_) => button_disable.set(false),
        }
    };

    use_effect(move || match (is_following(), mouse_hover()) {
        (true, false) => {
            button_icon.set("".to_string());
            button_text.set(format!(
                " Following {} ",
                article_detail().article.author.username
            ));
            button_class.set("text-yellow-500".to_string());
        }
        (true, true) => {
            button_icon.set("fa-solid fa-person-circle-minus w-4 h-4".to_string());
            button_text.set(format!(
                " Unfollow {} ",
                article_detail().article.author.username
            ));
            button_class.set("text-yellow-400".to_string());
        }
        (false, false) => {
            button_icon.set("fa-solid fa-person-circle-plus w-4 h-4".to_string());
            button_text.set(format!(
                " Follow {} ",
                article_detail().article.author.username
            ));
            button_class.set("".to_string());
        }
        (false, true) => {
            button_icon.set("fa-solid fa-person-circle-plus w-4 h-4".to_string());
            button_text.set(format!(
                " Follow {} ",
                article_detail().article.author.username
            ));
            button_class.set("hover:text-yellow-400".to_string());
        }
    });

    rsx! {
        form {
            onsubmit: on_submit,
            onmouseenter: move |_| mouse_hover.set(true),
            onmouseleave: move |_| mouse_hover.set(false),
            div { class: "rounded-md",
                button { disabled: button_disable(), class: button_class(),
                    i { class: button_icon() }
                    span { {button_text()} }
                }
            }
        }
    }
}
