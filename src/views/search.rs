use dioxus::prelude::*;

#[tracing::instrument]
#[post("/api/search_fetch_results")]
pub async fn search_fetch_results(
    search: String,
    page: i64,
    amount: i64,
) -> Result<((i64, i64, i64), Vec<crate::models::MatchedArticles>), ServerFnError> {
    dioxus::logger::tracing::info!("Starting search op");
    if search.is_empty() {
        Err(ServerFnError::new("Empty search string, hence ignore"))
    } else {
        let total = sqlx::query!(
            r#"SELECT
            COUNT(*) as "tot: i64" FROM Articles_fts AS AFTS JOIN  Articles AS A  ON A.oid = AFTS.rowid WHERE Articles_fts MATCH $1"#, search
        )
        .map(|x|x.tot)
        .fetch_one(crate::database::server::get_db())
        .await.map_err(|e| ServerFnError::new(format!("Some problem occured in sqlite query - {}", e.to_string())))
        ;

        Ok((
            (total.unwrap_or_default(), page, amount),
            crate::models::MatchedArticles::search_articles(search, page, amount)
                .await
                .map_err(|x| {
                    tracing::error!("problem while fetching search articles: {x:?}");
                    ServerFnError::new("Problem while fetching search articles")
                })?,
        ))
    }
}

use crate::{
    components::{AuthorUserIcon, ButtonFav, SearchViewPrevNextButton},
    models::MatchedArticles,
    views::article::CommentSection,
    SearchMeta, SearchString, SearchWindow,
};

#[component]
pub fn SearchResults(search_string_input: Signal<String>, hide_all: Signal<bool>) -> Element {
    let mut search_meta = use_context::<Signal<SearchMeta>>();
    let mut search_window = use_context::<Signal<SearchWindow>>();
    let mut search_string = use_context::<Signal<SearchString>>();
    let open_article_cnt = use_signal(|| 0);
    let mut search_result = use_resource(move || async move {
        search_fetch_results(search_string().0, search_meta().page, search_meta().amount).await
    });

    rsx! {

        match &*search_result.read() {
            Some(Ok(results)) => {
                let ((total_count, page, amount), _) = *results;
                rsx! {
                    if search_window().0 {
                        div { class: "flex justify-between mb-1",
                            div { class: "font-bold",
                                "Search results = "
                                {total_count.to_string()}
                            }
                            div {
                                if total_count > 0 {
                                    "  Showing results "
                                    {
                                        match page {
                                            0 => 1.to_string(),
                                            _ => (page * amount + 1).to_string(),
                                        }
                                    }
                                    " to "
                                    {
                                        match (page, total_count < amount) {
                                            (0, true) => total_count.to_string(),
                                            (0, false) => amount.to_string(),
                                            (_, _) => (std::cmp::min((page + 1) * amount, total_count)).to_string(),
                                        }
                                    }
                                    " of "
                                    {total_count.to_string()}
                                }
                                button {
                                    class: "text-blue-400 hover:underline hover:text-blue-500 transition duration-200 cursor-pointer px-8",
                                    onclick: move |_| {
                                        search_meta.set(SearchMeta { page, amount });
                                        search_window.set(SearchWindow(false));
                                        search_string.set(SearchString(String::new()));
                                        search_string_input.set(String::new());
                                        search_result.clear();
                                    },
                                    "Clear Search"
                                }
                            }
                            SearchViewPrevNextButton { page_data: (total_count, page, amount) }
                        }
                    }
                    for article in results.1.iter() {
                        SearchViewList { article: article.clone(), open_article_cnt, hide_all }
                    }
                    SearchViewPrevNextButton { page_data: (total_count, page, amount) }
                }
            }
            Some(Err(e)) => rsx! {
                div { class: "text-gray-800 dark:text-gray-200", "Failed to load: {e}" }
            },
            None => rsx! {
                div { class: "text-gray-800 dark:text-gray-200", "Loading Articles..." }
            },
        }
    }
}

#[component]
fn SearchViewList(
    article: MatchedArticles,
    open_article_cnt: Signal<i32>,
    hide_all: Signal<bool>,
) -> Element {
    let mut show_article = use_signal(|| false);
    use_effect(move || {
        if hide_all() {
            show_article.set(false);
        }
    });

    rsx! {
        div { class: "mb-2 p-4 bg-white dark:bg-gray-800 rounded-lg shadow-md text-gray-800 dark:text-gray-200",
            p {
                span { class: "font-bold", "Title: " }
                span { dangerous_inner_html: article.title.clone() }
            }
            p {
                span { class: "font-bold", "Description: " }
                span { dangerous_inner_html: article.description.clone() }
            }
            p {
                span { class: "font-bold", "Body: " }
                span { dangerous_inner_html: article.body.clone() }
            }
            div { class: "flex justify-between",
                div {
                    a {
                        href: format!("/article/{}", article.slug),
                        class: "text-blue-600 underline cursor-pointer",
                        target: "_blank",

                        "Open in a new tab/window"
                    }
                }
                div {
                    button {
                        class: "text-blue-600 underline cursor-pointer",
                        r#type: "button",
                        onclick: move |_| {
                            show_article.toggle();
                            match show_article() {
                                true => {
                                    open_article_cnt.set(open_article_cnt() + 1);
                                    hide_all.set(false);
                                }
                                false => open_article_cnt.set(open_article_cnt() - 1),
                            }
                        },
                        {format!("{}", if show_article() { "Hide" } else { "Show Full Article" })}
                    }
                    div {
                        if open_article_cnt() > 1 && show_article() {
                            button {
                                class: "text-blue-600 underline cursor-pointer",
                                r#type: "button",
                                onclick: move |_| {
                                    hide_all.set(true);
                                    open_article_cnt.set(0);
                                },
                                "Hide All"
                            }
                        }
                    }
                }
            }
            if show_article() && !hide_all() {
                SearchArticleView { slug: article.slug.clone() }
            }
        }
    }
}

#[component]
fn SearchArticleView(slug: ReadSignal<String>) -> Element {
    let article_resource =
        use_resource(move || async move { crate::views::article::get_article(slug()).await });

    rsx! {
        match &*article_resource.read() {
            Some(Ok(article_detail)) => rsx! {
                div { class: "bg-opacity-60 inset-0 flex items-center justify-center",
                    div { class: "block w-4/5 rounded-lg bg-white dark:bg-gray-800 p-4 shadow-[0_2px_15px_-3px_rgba(0,0,0,0.07),0_10px_20px_-2px_rgba(0,0,0,0.04)]",
                        div { class: "mb-5 px-1 py-1",
                            div { class: "mb-5",
                                ArticleMetaSearchView { article_detail: (*article_detail).clone() }
                            }
                            div { class: "flex justify-between mb-5",
                                div {
                                    div { class: "mb-2",
                                        h1 { class: "text-xl leading-tight font-medium text-neutral-800 dark:text-gray-200",
                                            {article_detail.article.title.clone()}
                                        }
                                    }
                                }
                            }
                            div { class: "mb-5",
                                p { class: "text-gray-800 dark:text-gray-300", {article_detail.article.body.clone()} }
                            }
                        }
                        div { class: "mb-5 px-1 py-1",
                            CommentSection {
                                article_detail: (*article_detail).clone(),
                                article_resource,
                            }
                        }
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
fn ArticleMetaSearchView(article_detail: ReadSignal<super::ArticleDetailed>) -> Element {
    rsx! {
        div {
            div { class: "flex items-center gap-4 text-gray-700 dark:text-gray-300",
                AuthorUserIcon { user: article_detail().article.author }
                div { class: "flex items-center gap-1",
                    span {
                        i { class: "fa-solid fa-calendar w-4 h-4" }
                    }
                    {article_detail().article.created_at}
                }
                div { class: "flex items-center gap-1",
                    i {
                        class: format!(
                            "{}",
                            if article_detail().article.comments_count > 0 {
                                "fas fa-comments w-4 h-4 text-yellow-500"
                            } else {
                                "far fa-comments w-4 h-4"
                            },
                        ),
                    }
                    span { class: "px-1",
                        " Comments: "
                        {article_detail().article.comments_count.to_string()}
                    }
                }

                ButtonFav { article_detail }
            }
        }
    }
}
