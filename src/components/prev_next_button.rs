use crate::models::Pagination;
use dioxus::prelude::*;

#[component]
pub fn PrevNextButton(
    articles: ReadOnlySignal<Vec<crate::models::Article>>,
    route_path: ReadOnlySignal<String>,
) -> Element {
    let mut pagination = use_context::<Signal<Pagination>>();
    let nav = navigator();
    let on_click_prev = move |_| {
        nav.push(format!(
            "{}{}",
            route_path(),
            pagination().previous_page().to_string()
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

    let on_click_next = move |_| {
        nav.push(format!(
            "{}{}",
            route_path(),
            pagination().next_page().to_string()
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
        if pagination().get_page() > 0 {
            button {
                onclick: on_click_prev,
                r#type: "button",
                class: "px-4 cursor-pointer hover:text-blue-500 border rounded-full bg-gray-100 text-gray-800",
                "<< Previous page      "
            }
        }
        if articles().len() > 0 && (articles().len() >= pagination().get_amount() as usize) {
            button {
                onclick: on_click_next,
                r#type: "button",
                class: "px-4 cursor-pointer hover:text-blue-500 border rounded-full bg-gray-100 text-gray-800",
                "Next page >>"
            }
        }
    }
}

#[component]
pub fn SearchViewPrevNextButton(page_data: (i64, i64, i64)) -> Element {
    let (total_count, page, amount) = page_data;
    let mut search_meta = use_context::<Signal<crate::SearchMeta>>();

    rsx! {
        div { class: "flex gap-2 justify-end",
            if page > 0 {
                button {
                    class: "px-4 cursor-pointer rounded-full border hover:text-blue-500",
                    onclick: move |_| {
                        search_meta
                            .set(crate::SearchMeta {
                                page: page - 1,
                                amount,
                            });
                    },
                    "Prev page"
                }
            }
            if match page {
                0 => total_count > amount,
                _ => total_count > ((page + 1) * amount),
            }
            {
                div {
                    button {
                        class: "px-4 cursor-pointer rounded-full border hover:text-blue-500",
                        onclick: move |_| {
                            search_meta
                                .set(crate::SearchMeta {
                                    page: page + 1,
                                    amount,
                                });
                        },
                        "Next page"
                    }
                }
            }
        }
    }
}
