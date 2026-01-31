use dioxus::prelude::*;

#[component]
pub fn AuthorUserIcon(user: crate::models::UserPreview) -> Element {
    rsx! {
        div { class: "flex gap-4 text-gray-600 dark:text-gray-400",
            a { href: format!("/profile/{}", user.username.clone()),
                img { class: "w-10 h-10 rounded-full", src: user.image }
            }
            div { class: "flex items-center",
                a { href: format!("/profile/{}", user.username.clone()),
                    i { class: "fa-solid fa-user w-4 h-4" }
                    span { class: "font-medium", {user.username.clone()} }
                }
            }

        }
    }
}

#[component]
pub fn CommentUserIcon(comment: crate::models::Comment) -> Element {
    rsx! {
        div { class: "flex gap-4 text-gray-600 dark:text-gray-400",
            div {
                a { href: format!("/profile/{}", comment.username.clone()),
                    img {
                        src: comment.user_image,
                        class: "w-10 h-10 rounded-full",
                    }
                }
            }
            div {
                i { class: "fa-solid fa-user w-4 h-4" }
                a { href: format!("/profile/{}", comment.username.clone()),
                    span { class: "font-medium", {comment.username.clone()} }
                }
            }
        }
    }
}

#[component]
pub fn CurrentUserIcon(article_detail: ReadSignal<crate::views::ArticleDetailed>) -> Element {
    rsx! {
        div { class: "flex gap-4 text-gray-600 dark:text-gray-400",
            div {
                a { href: format!("/profile/{}", article_detail().logged_user.unwrap_or_default().username()),
                    img {
                        src: article_detail().logged_user.unwrap_or_default().image(),

                        class: "w-10 h-10 rounded-full",
                    }
                }
            }
            div {
                i { class: "fa-solid fa-user w-4 h-4" }
                a { href: format!("/profile/{}", article_detail().logged_user.unwrap_or_default().username()),
                    span { class: "font-medium",
                        {article_detail().logged_user.unwrap_or_default().username()}
                    }
                }
            }
        }
    }
}
