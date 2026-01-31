use dioxus::prelude::*;

#[cfg(feature = "server")]
use dioxus::fullstack::{Cookie, TypedHeader};

#[derive(serde::Deserialize, Clone, serde::Serialize)]
pub enum EditorResponse {
    ValidationError(String),
    UpdateError,
    AuthError(String),
    Successful(String),
}
#[cfg(feature = "server")]
#[derive(Debug)]
struct ArticleUpdate {
    title: String,
    description: String,
    body: String,
    tag_list: std::collections::HashSet<String>,
}

const TITLE_MIN_LENGTH: usize = 4;
const DESCRIPTION_MIN_LENGTH: usize = 4;
const BODY_MIN_LENGTH: usize = 10;

#[cfg(feature = "server")]
#[tracing::instrument]
fn validate_article(
    title: String,
    description: String,
    body: String,
    tag_list: String,
) -> Result<ArticleUpdate, String> {
    if title.len() < TITLE_MIN_LENGTH {
        return Err("You need to provide a title with at least 4 characters".into());
    }

    if description.len() < DESCRIPTION_MIN_LENGTH {
        return Err("You need to provide a description with at least 4 characters".into());
    }

    if body.len() < BODY_MIN_LENGTH {
        return Err("You need to provide a body with at least 10 characters".into());
    }

    let tag_list = tag_list
        .trim()
        .split_ascii_whitespace()
        .filter(|x| !x.is_empty())
        .map(str::to_string)
        .collect::<std::collections::HashSet<String>>();
    Ok(ArticleUpdate {
        title,
        description,
        body,
        tag_list,
    })
}

#[cfg(feature = "server")]
#[tracing::instrument]
async fn update_article(
    author: String,
    slug: String,
    article: ArticleUpdate,
) -> Result<String, sqlx::Error> {
    static BIND_LIMIT: usize = 65535;
    let mut transaction = crate::database::server::get_db().begin().await?;
    let (rows_affected, slug) = if !slug.is_empty() {
        (
            sqlx::query!(
                "UPDATE Articles SET title=$1, description=$2, body=$3 WHERE slug=$4 and author=$5",
                article.title,
                article.description,
                article.body,
                slug,
                author,
            )
            .execute(transaction.as_mut())
            .await?
            .rows_affected(),
            slug.to_string(),
        )
    } else {
        let slug = uuid::Uuid::now_v7().to_string();
        (sqlx::query!(
            "INSERT INTO Articles(slug, title, description, body, author) VALUES ($1, $2, $3, $4, $5)",
            slug,
            article.title,
            article.description,
            article.body,
            author
        )
        .execute(transaction.as_mut())
        .await?.rows_affected(),
        slug)
    };
    if rows_affected != 1 {
        // We are going to modify just one row, otherwise something funky is going on
        tracing::error!("no rows affected");
        return Err(sqlx::Error::RowNotFound);
    }
    sqlx::query!("DELETE FROM ArticleTags WHERE article=$1", slug)
        .execute(transaction.as_mut())
        .await?;
    if !article.tag_list.is_empty() {
        let mut qb = sqlx::QueryBuilder::new("INSERT INTO ArticleTags(article, tag) ");
        qb.push_values(
            article.tag_list.clone().into_iter().take(BIND_LIMIT / 2),
            |mut b, tag| {
                b.push_bind(slug.clone()).push_bind(tag);
            },
        );
        qb.build().execute(transaction.as_mut()).await?;
    }

    transaction.commit().await?;
    Ok(slug)
}

#[tracing::instrument]
#[post("/api/editor_action", header: TypedHeader<Cookie>)]
pub async fn editor_action(
    title: String,
    description: String,
    body: String,
    tag_list: String,
    slug: String,
) -> Result<EditorResponse, ServerFnError> {
    let Some(author) = crate::auth::get_username_from_cookie(header) else {
        return Ok(EditorResponse::AuthError(
            "you should be authenticated".to_string(),
        ));
    };
    let article = match validate_article(title, description, body, tag_list) {
        Ok(x) => x,
        Err(x) => return Ok(EditorResponse::ValidationError(x)),
    };
    match update_article(author, slug, article).await {
        Ok(x) => {
            // crate::server_fn::redirect::call_redirect_hook("/article/{x}");
            let _ = dioxus_fullstack::Redirect::to("/article/{x}");

            Ok(EditorResponse::Successful(x))
        }
        Err(x) => {
            tracing::error!("EDITOR ERROR: {}", x.to_string());
            Ok(EditorResponse::UpdateError)
        }
    }
}

#[component]
pub fn Editor(slug: String) -> Element {
    let nav = navigator();
    let mut editor_status = use_signal(|| "".to_string());
    let mut edit_article = use_signal(|| super::article::ArticleDetailed::default());

    if !slug.is_empty() {
        let _ = use_resource(move || {
            let value = slug.clone();
            async move {
                match super::article::get_article(value).await {
                    Ok(res) => {
                        edit_article.set(res);
                    }

                    Err(err) => {
                        tracing::error!("Error returned while get_article : {}", err.to_string());
                    }
                }
            }
        });
    }

    let on_submit = move |evt: FormEvent| async move {
        evt.prevent_default();
        let title_data = evt.values().into_iter().filter(|d| d.0 == "title").last();
        let title = match title_data {
            Some((_, FormValue::Text(value))) => value,
            _ => String::new(),
        };
        let description_data = evt
            .values()
            .into_iter()
            .filter(|d| d.0 == "description")
            .last();
        let description = match description_data {
            Some((_, FormValue::Text(value))) => value,
            _ => String::new(),
        };
        let body_data = evt.values().into_iter().filter(|d| d.0 == "body").last();
        let body = match body_data {
            Some((_, FormValue::Text(value))) => value,
            _ => String::new(),
        };

        let tags_data = evt
            .values()
            .into_iter()
            .filter(|d| d.0 == "tag_list")
            .last();
        let tags = match tags_data {
            Some((_, FormValue::Text(value))) => value,
            _ => String::new(),
        };

        let res = editor_action(title, description, body, tags, edit_article().article.slug).await;

        match res {
            Ok(EditorResponse::Successful(_)) => {
                nav.replace(crate::Route::Home {});
            }
            Ok(EditorResponse::ValidationError(e)) => {
                editor_status.set(e);
            }
            Ok(EditorResponse::UpdateError) => (),
            Ok(EditorResponse::AuthError(_)) => {
                nav.replace(crate::Route::Login {});
            }
            Err(e) => {
                editor_status.set(e.to_string());
            }
        }
    };

    rsx! {
        div { class: "fixed inset-0 z-50 flex items-center justify-center bg-gray-900 bg-opacity-60",
            document::Title { "Create a new Article" }
            div { class: "block rounded-lg bg-white dark:bg-gray-800 w-2/5 p-4 shadow-[0_2px_15px_-3px_rgba(0,0,0,0.07),0_10px_20px_-2px_rgba(0,0,0,0.04)] z-70",
                div { class: "col-md-10 offset-md-1 col-xs-12",
                    form { id: "editor", onsubmit: on_submit,
                        div { class: "mb-5",
                            input {
                                name: "title",
                                r#type: "text",
                                class: "shadow appearance-none border dark:border-gray-600 rounded w-full py-2 px-3 text-gray-700 dark:text-gray-200 dark:bg-gray-700 leading-tight focus:ring",
                                placeholder: "Article Title",
                                minlength: TITLE_MIN_LENGTH,
                                value: edit_article().article.title,
                            }
                        }
                        div { class: "mb-5",
                            input {
                                name: "description",
                                r#type: "text",
                                class: "shadow appearance-none border dark:border-gray-600 rounded w-full py-2 px-3 text-gray-700 dark:text-gray-200 dark:bg-gray-700 leading-tight focus:ring",
                                placeholder: "What's this article about?",
                                minlength: DESCRIPTION_MIN_LENGTH,
                                value: edit_article().article.description,
                            }
                        }
                        div { class: "mb-5",
                            textarea {
                                name: "body",
                                rows: 8,
                                class: "shadow appearance-none border dark:border-gray-600 rounded w-full py-2 px-3 text-gray-700 dark:text-gray-200 dark:bg-gray-700 leading-tight focus:ring",
                                placeholder: "Write your article (in markdown)",
                                minlength: BODY_MIN_LENGTH,
                                value: edit_article().article.body,
                            }
                        }
                        div { class: "mb-5",
                            input {
                                class: "shadow appearance-none border dark:border-gray-600 rounded w-full py-2 px-3 text-gray-700 dark:text-gray-200 dark:bg-gray-700 leading-tight focus:ring",
                                name: "tag_list",
                                placeholder: "Enter tags(space separated)",
                                r#type: "text",
                                value: edit_article().article.tag_list.join(" "),
                            }
                        }

                        div { class: "flex flex-row-reverse space-x-4 space-x-reverse",

                            button { class: "bg-blue-700 hover:bg-blue-800 px-5 py-3 text-white rounded-lg",
                                "Publish Article"
                            }
                            button {
                                class: "bg-gray-300 hover:bg-gray-400 dark:bg-gray-600 dark:hover:bg-gray-500 px-5 py-3 text-white rounded-lg",
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
                        }
                        div { class: "text-red-600", {editor_status()} }
                    }
                }
            }
        }
    }
}
