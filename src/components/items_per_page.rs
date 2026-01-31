use crate::models::Pagination;
use crate::PageAmount;
use dioxus::prelude::*;
use dioxus::prelude::{component, rsx, Element};
use dioxus::router::root_router;

#[component]
pub fn ItemsPerPage(route_path: ReadSignal<String>) -> Element {
    let mut pagination = use_context::<Signal<Pagination>>();
    let mut page_amount = use_context::<Signal<crate::PageAmount>>();

    let nav = navigator();

    let on_change = move |ev: FormEvent| {
        let amount = ev.value().parse::<i64>().unwrap_or(10);
        page_amount.set(PageAmount(amount));
        nav.push(format!(
            "{}{}",
            route_path(),
            pagination().set_amount(amount).reset_page().to_string()
        ));
        let router_context = root_router();
        let mut route_string = String::new();
        if let Some(context) = router_context {
            route_string = context.full_route_string()
        }
        if pagination() != Pagination::from(route_string.clone()) {
            pagination.set(Pagination::from(route_string.clone()));
        }
        spawn(async move {
            let _ = crate::auth::update_per_page_amount(amount as u32).await;
        });
    };
    rsx! {
        div {
            label { class: "text-gray-700 dark:text-gray-300 px-1", r#for: "articlesPerPage", "Items Per Page" }
            select {
                id: "articlesPerPage",
                class: "focus:shadow-outline rounded border dark:border-gray-600 px-1 py-1 leading-tight text-gray-700 dark:text-gray-200 dark:bg-gray-700 shadow focus:outline-none",
                onchange: on_change,
                option { id: "1", selected: page_amount().0 == 1, value: "1", "  1" }
                option { id: "5", selected: page_amount().0 == 5, value: "5", "  5" }
                option { id: "10", selected: page_amount().0 == 10, value: "10", " 10" }
                option { id: "20", selected: page_amount().0 == 20, value: "20", " 20" }
                option {
                    id: "100",
                    selected: page_amount().0 == 100,
                    value: "100",
                    "100"
                }
            }
        }
    }
}
