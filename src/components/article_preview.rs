use crate::components::{AuthorUserIcon, ButtonFav, ButtonFavFavourited, ButtonFollow};
use crate::models::article::Article;
use crate::models::{Pagination, User};
#[cfg(feature = "server")]
use dioxus::fullstack::{Cookie, TypedHeader};
use dioxus::prelude::*;
use dioxus::router::root_router;

#[component]
pub fn ArticlePreviewList(
    articles: ReadSignal<Vec<Article>>,
    logged_user: Option<User>,
) -> Element {
    let mut pagination = use_context::<Signal<Pagination>>();
    let nav = navigator();

    let on_submit = move |evt: FormEvent| {
        evt.prevent_default();
        let tag_data = evt.values().into_iter().filter(|d| d.0 == "tag").last();
        let tag = match tag_data {
            Some((_, FormValue::Text(tag))) => tag,
            _ => String::new(),
        };
        nav.push(pagination().set_tag(&tag).to_string());
        let router_context = root_router();
        let mut route_string = String::new();
        if let Some(context) = router_context {
            route_string = context.full_route_string()
        }
        pagination.set(Pagination::from(route_string.clone()));
    };

    rsx! {
        for article in articles.iter() {
            div { class: "mb-2 p-4 bg-white rounded-lg shadow-md",
                div { class: "flex items-center gap-4 mb-4",
                    ArticleMeta {
                        article_detail: crate::views::ArticleDetailed {
                            article: article.clone(),
                            logged_user: logged_user.clone(),
                        },
                        is_preview: true,
                    }
                }

                a { href: "/article/{article.slug.clone()}",
                    h2 { class: "text-2xl font-bold mb-2 text-gray-800", {article.title.clone()} }
                }

                a { href: "/article/{article.slug.clone()}",
                    p { class: "text-gray-700 mb-4", {article.description.clone()} }
                }

                div { class: "flex justify-between items-end text-gray-600",
                    a { href: "/article/{article.slug.clone()}",
                        span { class: "hover:text-blue-600 hover:underline cursor-pointer",
                            "Read more..."
                        }
                    }

                    div { class: "flex flex-wrap gap-1",
                        i { class: "fa-solid fa-hashtag py-1" }
                        for tag in article.tag_list.clone() {
                            if !tag.is_empty() {
                                form { onsubmit: on_submit,
                                    input {
                                        r#type: "hidden",
                                        name: "tag",
                                        value: tag.clone(),
                                    }
                                    button {
                                        span { class: "bg-gray-200 text-gray-700 px-2 py-1 rounded text-xs flex items-center gap-1 cursor-pointer",
                                            {tag.clone()}
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn ArticleMeta(
    article_detail: ReadSignal<crate::views::ArticleDetailed>,
    is_preview: bool,
) -> Element {
    let is_owner = use_signal(|| {
        article_detail().article.author.username
            == article_detail()
                .logged_user
                .clone()
                .unwrap_or_default()
                .username
    });
    let nav = navigator();
    let pagination = use_context::<Signal<Pagination>>();
    let mut delete_status = use_signal(|| String::new());
    let on_submit = move |evt: FormEvent| async move {
        evt.prevent_default();
        let res = delete_article(article_detail().article.slug).await;

        match res {
            Ok(_) => {
                nav.replace(crate::Route::Home {});
            }
            Err(err) => {
                delete_status.set(format!("Some error occured! : {}", err.to_string()));
            }
        }
    };

    rsx! {
        div { class: "flex items-center gap-4 text-gray-700",

            if is_preview {
                AuthorUserIcon { user: article_detail().article.author.clone() }
            }
            div { class: "flex items-center gap-1",

                i { class: "fa-solid fa-calendar w-4 h-4" }
                span { class: "", {article_detail().article.created_at} }
            }
            div { class: "flex items-center gap-1",

                i {
                    class: format!(
                        "{}",
                        if article_detail().article.comments_count > 0 {
                            "fas fa-comments w-4 h-4 text-yellow-500"
                        } else {
                            "far fa-comments w-4 h-4 text-gray-500"
                        },
                    ),
                }
                span { class: "px-1",
                    " Comments: "
                    {article_detail().article.comments_count.to_string()}
                }
            }
            if is_preview {
                if pagination().get_favourites() {
                    ButtonFavFavourited { article_detail }
                } else {
                    ButtonFav { article_detail }
                }
            } else {
                if is_owner() {
                    div { class: "",
                        a { href: "/editor/{article_detail().article.slug}",
                            i { class: "fa-solid fa-pen-to-square w-4 h-4" }
                            span { " Edit article" }
                        }
                    }

                    div { class: "text-red-400 hover:rounded hover:border hover:bg-red-100",
                        form { onsubmit: on_submit,
                            button {
                                i { class: "fa-solid fa-trash-can w-4 h-4" }
                                span { " Delete Article" }
                            }
                            p { class: "text-red-700", {delete_status()} }
                        }
                    }
                } else {
                    if article_detail().logged_user.is_some() {
                        ButtonFav { article_detail }
                        ButtonFollow { article_detail }
                    } else {
                        ButtonFav { article_detail }
                    }
                }
            }
        }
    }
}

#[tracing::instrument]
#[post("/api/delete_article", header: TypedHeader<Cookie>)]
pub async fn delete_article(slug: String) -> Result<bool, ServerFnError> {
    let Some(logged_user) = crate::auth::get_username_from_cookie(header) else {
        return Err(ServerFnError::new("you must be logged in"));
    };

    crate::models::Article::delete(slug, logged_user)
        .await
        .map(|_| true)
        .map_err(|x| {
            let err = format!("Error while deleting an article: {x:?}");
            tracing::error!("{err}");
            ServerFnError::new("Could not delete the article, try again later")
        })
}
