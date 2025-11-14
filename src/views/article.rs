use crate::components::{AuthorUserIcon, CommentUserIcon, CurrentUserIcon};
use dioxus::prelude::*;

use crate::components::ArticleMeta;

#[derive(serde::Deserialize, serde::Serialize, Clone, Default)]
pub struct ArticleDetailed {
    pub article: crate::models::Article,
    pub logged_user: Option<crate::models::User>,
}

#[server]
#[tracing::instrument]
pub async fn get_article(slug: String) -> Result<ArticleDetailed, ServerFnError> {
    let server_context = server_context();
    let request_parts: axum::http::request::Parts = server_context.extract().await?;

    Ok(ArticleDetailed {
        article: crate::models::Article::for_article(slug, request_parts)
            .await
            .map_err(|x| {
                let err = format!("Error while getting user_profile articles: {x:?}");
                tracing::error!("{err}");
                ServerFnError::new("Could not retrieve articles, try again later")
            })?,
        logged_user: crate::auth::current_user().await.ok(),
    })
}

#[server]
#[tracing::instrument]
pub async fn post_comment(slug: String, body: String) -> Result<(), ServerFnError> {
    let server_context = server_context();
    let request_parts: axum::http::request::Parts = server_context.extract().await?;

    let Some(logged_user) = crate::auth::get_username(request_parts) else {
        return Err(ServerFnError::ServerError("you must be logged in".into()));
    };

    crate::models::Comment::insert(slug, logged_user, body)
        .await
        .map(|_| ())
        .map_err(|x| {
            let err = format!("Error while posting a comment: {x:?}");
            tracing::error!("{err}");
            ServerFnError::ServerError("Could not post a comment, try again later".into())
        })
}

#[server]
#[tracing::instrument]
pub async fn get_comments(slug: String) -> Result<Vec<crate::models::Comment>, ServerFnError> {
    crate::models::Comment::get_all(slug).await.map_err(|x| {
        let err = format!("Error while posting a comment: {x:?}");
        tracing::error!("{err}");
        ServerFnError::ServerError("Could not post a comment, try again later".into())
    })
}

#[server]
#[tracing::instrument]
pub async fn delete_comment(id: i32) -> Result<(), ServerFnError> {
    let server_context = server_context();
    let request_parts: axum::http::request::Parts = server_context.extract().await?;
    let Some(logged_user) = crate::auth::get_username(request_parts) else {
        return Err(ServerFnError::ServerError("you must be logged in".into()));
    };

    crate::models::Comment::delete(id, logged_user)
        .await
        .map(|_| ())
        .map_err(|x| {
            let err = format!("Error while posting a comment: {x:?}");
            tracing::error!("{err}");
            ServerFnError::ServerError("Could not post a comment, try again later".into())
        })
}

#[component]
pub fn SearchArticle(slug: String) -> Element {
    rsx! {
        Article { slug, force_home: Some(true) }
    }
}

#[component]
pub fn Article(slug: ReadOnlySignal<String>, force_home: Option<bool>) -> Element {
    let article_resource = use_resource(move || {
        let value = slug();
        async move { get_article(value).await }
    });
    // .suspend()?;
    rsx! {
        match &*article_resource.read() {
            Some(Ok(article_detail)) => rsx! {


                div { class: "bg-opacity-60 inset-0 z-50 flex items-center justify-center",

                    div { class: "z-70 block w-4/5 rounded-lg bg-white p-4 shadow-[0_2px_15px_-3px_rgba(0,0,0,0.07),0_10px_20px_-2px_rgba(0,0,0,0.04)]",
                        div { class: "mb-5 px-1 py-1",

                            div { class: "mb-5",
                                ArticleMeta { article_detail: (*article_detail).clone(), is_preview: false }
                            }
                            div { class: "flex justify-between mb-5",
                                div {
                                    div { class: "mb-2",
                                        h1 { class: "text-xl leading-tight font-medium text-neutral-800",
                                            {article_detail.article.title.clone()}
                                        }
                                        document::Title { {article_detail.article.title.clone()} }
                                    }

                                    AuthorUserIcon { user: article_detail.article.author.clone() }
                                }
                                div {
                                    BackToButton { is_top: true, force_home }
                                }
                            }
                            div { class: "mb-5",
                                p { class: "text-neutral-800", {article_detail.article.body.clone()} }
                            }
                        }
                        div { class: "mb-5 px-1 py-1",
                            CommentSection { article_detail: (*article_detail).clone(), article_resource: article_resource }
                        }

                        BackToButton { is_top: false, force_home }
                    }
                }
            },
            Some(Err(e)) => rsx! {
            "Failed to load: {e}"
            },
            None => rsx! {
            "Loading article {slug}..."
            },
        }
    }
}

#[component]
fn BackToButton(is_top: bool, force_home: Option<bool>) -> Element {
    let nav = navigator();

    rsx! {
        div { class: "flex justify-end mb-5",
            button {
                r#type: "button",
                class: format!(
                    "fixed bg-blue-700 hover:bg-blue-800 px-15 py-3 text-white font-semibold rounded-lg transition-colors duration-300 {}",
                    if is_top { "top-0 left-0" } else { "bottom-4 right-4" },
                ),
                onclick: move |_| {
                    if nav.can_go_back() && force_home.unwrap_or_default() {
                        nav.go_back();
                    } else {
                        nav.push(crate::Route::Home {});
                    }
                },
                "Back"
            }
        }
    }
}

#[component]
pub fn CommentSection(
    article_detail: ReadOnlySignal<ArticleDetailed>,
    article_resource: Resource<Result<ArticleDetailed, ServerFnError>>,
) -> Element {
    let mut comment_post_status = use_signal(|| String::new());
    let mut comments_result = use_signal(|| vec![]);
    let mut new_comment_data = use_signal(|| String::new());

    let mut comments_fut = use_resource(move || {
        let value = article_detail().article.slug;
        async move {
            match get_comments(value).await {
                Ok(comments) => {
                    comments_result.set(comments);
                }
                Err(_) => (),
            }
        }
    });
    let on_submit = move |evt: FormEvent| async move {
        let res = post_comment(
            evt.values()["slug"].as_value(),
            evt.values()["body"].as_value(),
        )
        .await;

        match res {
            Ok(_) => {
                comments_fut.restart();
                article_resource.restart();
                new_comment_data.set(String::new());
            }
            Err(e) => {
                comment_post_status.set(format!("Status: {}", e.to_string()));
            }
        }
    };

    let on_submit_delete = move |evt: FormEvent| async move {
        let _ = delete_comment(evt.values()["id"].as_value().parse::<i32>().unwrap()).await;
        comments_fut.restart();
        article_resource.restart();
    };

    rsx! {
        div { class: "mb-1",
            form { onsubmit: on_submit,
                input {
                    name: "slug",
                    r#type: "hidden",
                    value: article_detail().article.slug,
                }
                h2 { class: "mb-2 block text-sm font-bold text-gray-700", "Comments" }
                div { class: "mb-1",
                    textarea {
                        class: "focus:shadow-outline w-full border-b appearance-none rounded px-3 py-2 leading-tight text-sm text-gray-700 shadow focus:outline-none",
                        name: "body",
                        placeholder: "Write a new comment...(min length 3 char)",
                        oninput: move |evt| new_comment_data.set(evt.value()),
                        value: "{new_comment_data}",
                    }
                }
                div { class: "flex mb-5",
                    CurrentUserIcon { article_detail }
                    div { class: "px-2",
                        button {
                            class: format!(
                                "rounded px-1 py-1 text-sm font-medium text-white {}",
                                if new_comment_data().len() < 3 {
                                    "bg-gray-300 cursor-not-allowed"
                                } else {
                                    "bg-blue-700 hover:bg-blue-800"
                                },
                            ),
                            "Post Comment"
                        }
                        p { class: "text-red-400", {comment_post_status()} }
                    }
                }
            }

            for comment in comments_result.iter() {

                div { class: "py-5",
                    CommentUserIcon { comment: comment.clone() }
                    div { class: "flex grow justify-between",
                        p { class: "text-neutral-800", {comment.body.clone()} }
                        div { class: "flex-none px-3 text-gray-600",

                            div {
                                i { class: "fa-solid fa-calendar w-4 h-4" }
                                span { class: "px-1", {comment.created_at.clone()} }
                            }

                           if {comment.username.clone()} == {article_detail().logged_user.unwrap_or_default().username()} {
                            form {
                                onsubmit: on_submit_delete,
                                input { r#type: "hidden", name: "id", value: comment.id }
                                button { class:"text-red-400 hover:rounded hover:border hover:bg-red-100", i { class:"fas fa-trash" } span { class:"px-1", "Delete" } }
                            }
                           }
                        }
                    }
                }
            }
        }
    }
}
