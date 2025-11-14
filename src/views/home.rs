use crate::components::{ItemsPerPage, PrevNextButton};
use crate::models::Pagination;
use crate::views::SearchResults;
use crate::{components::ArticlePreviewList, models::User};
use crate::{SearchString, SearchWindow};
use dioxus::events::FormEvent;
use dioxus::{document, prelude::*};

#[component]
pub fn Home() -> Element {
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
            document::Title { "Home Page" }
            HomePage { logged_user: logged_user().0, route_path: "".to_string() }
        }
    }
}

#[component]
pub fn HomePage(logged_user: Option<User>, route_path: ReadOnlySignal<String>) -> Element {
    let pagination = use_context::<Signal<Pagination>>();
    let search_window = use_context::<Signal<SearchWindow>>();
    let search_string_input = use_signal(|| String::new());
    let hide_all = use_signal(|| false);

    let articles_resource = use_resource(move || async move {
        home_articles(
            pagination().get_page(),
            pagination().get_amount(),
            pagination().get_tag().into(),
            pagination().get_my_feed(),
        )
        .await
    });

    rsx! {

        match &*articles_resource.read() {
            Some(Ok(articles)) => rsx! {
                div { class: "mx-auto sm:px-6 lg:px-8 bg-gray-200 px-2 py-2 sm:px-0 text-gray-800",
                    // div { class: "mx-auto max-w-7xl sm:px-6 lg:px-8 bg-gray-200 px-2 py-2 sm:px-0 text-gray-800",
                    div {
                        div { class: "flex justify-between text-gray-800",
                            div {
                                YourFeedTab { logged_user: logged_user.clone() }
                                GlobalFeedTab {}
                            }
                            div {
                                SearchArticle { search_string_input, hide_all }
                            }
                            ItemsPerPage { route_path: route_path() }
                        }
                        if search_window().0 {
                            SearchResults { search_string_input, hide_all }
                        } else {
                            if !pagination().get_my_feed() {
                                div { class: "flex gap-1 rounded bg-white mb-2",
                                    span { class: "font-bold m-1 text-gray-800", "Popular Tags:" }
                                    TagList {}
                                }
                            }
                            if articles.len() > 0 {
                                ArticlePreviewList { articles: articles.clone(), logged_user }
                            } else {
                                if pagination().get_my_feed() {
                                    div {
                                        p { class: "text-gray-700", "You are not following any other user!" }
                                    }
                                } else {
                                    div {
                                        p { class: "text-gray-700", "No articles to list" }
                                    }
                                }
                            }
                        }
                    }
                    if !search_window().0 {
                        div { class: "flex gap-4",
                            PrevNextButton { articles: articles.clone(), route_path: route_path() }
                        }
                    }
                    crate::Footer {}
                }
            },
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
fn SearchArticle(search_string_input: Signal<String>, hide_all: Signal<bool>) -> Element {
    let mut search_string = use_context::<Signal<SearchString>>();
    let mut search_window = use_context::<Signal<SearchWindow>>();

    rsx! {
        form {
            div { class: "flex justify-end",
                div { class: "flex justify-end",
                    input {
                        class: "shadow appearance-none bg-white border rounded w-full py-1 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline",
                        r#type: "text",
                        name: "search",
                        minlength: 2,
                        placeholder: "Search string",
                        required: true,
                        oninput: move |ev| search_string_input.set(ev.value()),
                        value: search_string_input(),
                    }
                    input { r#type: "hidden", name: "page", value: 0 }
                    input { r#type: "hidden", name: "amount", value: 10 }
                    button {
                        class: "absolute pr-3 cursor-pointer hover:text-blue-500 transition duration-200 py-1",
                        onclick: move |_| {
                            if !search_string_input().is_empty() {
                                search_string.set(SearchString(search_string_input()));
                                search_window.set(SearchWindow(true));
                                hide_all.set(true);
                            }
                        },
                        i { class: "fas fa-magnifying-glass" }
                    }
                }
            }
        }
    }
}

use crate::models::article::Article;

#[server]
pub async fn home_articles(
    page: i64,
    amount: i64,
    tag: String,
    my_feed: bool,
) -> Result<Vec<Article>, ServerFnError> {
    dioxus_logger::tracing::info!("Starting home_articles");
    let server_context = server_context();
    let request_parts: axum::http::request::Parts = server_context.extract().await?;

    Ok(
        Article::for_home_page(page, amount, tag, my_feed, request_parts)
            .await
            .map_err(|x| {
                tracing::error!("problem while fetching home articles: {x:?}");
                ServerFnError::new("Problem while fetching home articles")
            })?,
    )
}

#[component]
fn YourFeedTab(logged_user: Option<User>) -> Element {
    let mut pagination = use_context::<Signal<Pagination>>();
    let page_amount = use_context::<Signal<crate::PageAmount>>();

    let nav = navigator();

    let on_click = move |_| {
        nav.push(
            pagination()
                .set_my_feed(true)
                .reset_page()
                .set_amount(page_amount().0)
                .to_string(),
        );
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
        button {
            disabled: logged_user.is_none(),
            onclick: on_click,
            r#type: "button",
            class: format!(
                "px-1 m-1 font-bold disabled:cursor-not-allowed {}",
                if logged_user.is_none() {
                    "cursor-not-allowed bg-gray-200"
                } else if pagination().get_my_feed() {
                    "border-b-8 bg-gray-200"
                } else {
                    "bg-gray-200 cursor-pointer"
                },
            ),
            "Your Feed"
        }
    }
}

#[component]
fn GlobalFeedTab() -> Element {
    let mut pagination = use_context::<Signal<Pagination>>();
    let page_amount = use_context::<Signal<crate::PageAmount>>();

    let nav = navigator();
    let on_click = move |_| {
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
        button {
            onclick: on_click,
            r#type: "button",
            class: format!(
                "px-1 m-1 font-bold disabled:cursor-not-allowed {}",
                if !pagination().get_my_feed() {
                    "border-b-8 bg-gray-200"
                } else {
                    "bg-gray-200 cursor-pointer"
                },
            ),
            "Global Feed"
        }
    }
}

#[server]
async fn get_tags() -> Result<Vec<String>, ServerFnError> {
    // sqlx::query!("SELECT DISTINCT tag FROM ArticleTags")
    // instead of all tags, a union of most recent 10 tags + most repeated 10
    sqlx::query!(
        "
            SELECT
                tag,
                tag_count,
                max_created_at
            FROM (
                SELECT
                    T1.tag,
                    COUNT(T1.article) AS tag_count,
                    MAX(T2.created_at) AS max_created_at
                FROM
                    ArticleTags AS T1
                JOIN
                    Articles AS T2
                ON
                    T1.article = T2.slug
                GROUP BY
                    T1.tag
            	ORDER BY
            		tag_count DESC
            	LIMIT 10
            ) AS combined_tags

            UNION

            SELECT
                tag,
                tag_count,
                max_created_at
            FROM (
                SELECT
                    T1.tag,
                    COUNT(T1.article) AS tag_count,
                    MAX(T2.created_at) AS max_created_at
                FROM
                    ArticleTags AS T1
                JOIN
                    Articles AS T2
                ON
                    T1.article = T2.slug
                GROUP BY
                    T1.tag
                ORDER BY
                    max_created_at DESC
                LIMIT 10
            ) AS combined_tags
        "
    )
    .map(|x| x.tag)
    .fetch_all(crate::database::server::get_db())
    .await
    .map(|tags| {
        tags.into_iter()
            .filter_map(|tag| tag)
            .collect::<Vec<String>>()
    })
    .map_err(|x| {
        tracing::error!("problem while fetching tags: {x:?}");
        ServerFnError::ServerError("Problem while fetching tags".into())
    })
}

#[component]
fn TagList() -> Element {
    let mut pagination = use_context::<Signal<Pagination>>();
    let nav = navigator();

    let tag_list = use_resource(move || async move { get_tags().await });

    let on_submit = move |ev: FormEvent| {
        let tag = ev.values()["tag"].as_value();
        nav.push(pagination().set_tag(&tag).reset_page().to_string());
        let router_context = root_router();
        let mut route_string = String::new();
        if let Some(context) = router_context {
            route_string = context.full_route_string()
        }
        pagination.set(Pagination::from(route_string.clone()));
    };
    let tag_view = {
        let current_pagination = pagination();
        let tag_elected = current_pagination.get_tag();

        match &*tag_list.read() {
            Some(Ok(tags)) => {
                rsx! {
                    for tag in tags.iter() {
                        form { onsubmit: on_submit,
                            div { class: "gap-1",
                                input {
                                    r#type: "hidden",
                                    name: "tag",
                                    value: if tag == &tag_elected { "".to_string() } else { (&tag.clone()).to_string() },
                                }
                                button {
                                    class: format!(
                                        "rounded px-1 py-0.5 hover:bg-green-300 text-gray-800 cursor-pointer {}",
                                        if tag == &tag_elected { "bg-green-200" } else { "bg-gray-200" },
                                    ),
                                    {tag.clone()}
                                }
                            }
                        }
                    }
                }
            }
            Some(Err(e)) => rsx! {
                div { class: "text-gray-800", "Failed to load: {e}" }
            },
            None => rsx! {
                div { class: "text-gray-800", "Loading Tags..." }
            },
        }
    };
    rsx! {
        div { class: "flex gap-1 py-1 flex-wrap", {tag_view} }
    }
}
