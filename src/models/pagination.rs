#[derive(Debug, PartialEq, Clone)]
pub struct Pagination {
    tag: Option<String>,
    my_feed: Option<bool>,
    page: Option<i64>,
    amount: Option<i64>,
    favourites: Option<bool>,
}

impl Pagination {
    #[inline]
    pub fn get_tag(&self) -> &str {
        self.tag.as_deref().unwrap_or_default()
    }
    #[inline]
    pub fn get_my_feed(&self) -> bool {
        self.my_feed.unwrap_or_default()
    }
    #[inline]
    pub fn get_favourites(&self) -> bool {
        self.favourites.unwrap_or_default()
    }
    #[inline]
    pub fn get_page(&self) -> i64 {
        self.page.unwrap_or_default()
    }
    #[inline]
    pub fn get_amount(&self) -> i64 {
        self.amount.unwrap_or_default()
    }

    #[inline]
    pub fn set_tag<T: ToString + ?Sized>(mut self, tag: &T) -> Self {
        self.tag = Some(tag.to_string());
        self
    }

    #[inline]
    pub fn set_amount(mut self, amount: i64) -> Self {
        self.amount = Some(amount);
        self
    }

    #[inline]
    pub fn set_my_feed(mut self, feed: bool) -> Self {
        self.my_feed = Some(feed);
        self
    }

    #[inline]
    pub fn set_favourites(mut self, feed: bool) -> Self {
        self.favourites = Some(feed);
        self
    }

    #[inline]
    pub fn reset_page(mut self) -> Self {
        self.page = Some(0);
        self
    }

    #[inline]
    pub fn next_page(mut self) -> Self {
        self.page = Some(self.page.unwrap_or_default().saturating_add(1));
        self
    }

    #[inline]
    pub fn previous_page(mut self) -> Self {
        self.page = Some(self.page.unwrap_or_default().saturating_sub(1));
        self
    }
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            tag: Some(String::new()),
            my_feed: Some(false),
            page: Some(0),
            amount: Some(10),
            // amount: Some(page_amount().0),
            favourites: Some(false),
        }
    }
}

impl std::fmt::Display for Pagination {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "/?tag={}&my_feed={}&page={}&amount={}&favourites={}",
            self.get_tag(),
            self.get_my_feed(),
            self.get_page(),
            self.get_amount(),
            self.get_favourites(),
        )
    }
}

impl From<String> for Pagination {
    fn from(url: String) -> Self {
        let mut tag = Some(String::new());
        let mut my_feed = Some(false);
        let mut page = Some(0);
        let mut amount = Some(10);
        let mut favourites = Some(false);

        for param in url.split('&') {
            // dioxus::logger::tracing::info!("Param is: {param}");
            if let Some((key, value)) = param.split_once('=') {
                match key.trim_start_matches("/?") {
                    "tag" => tag = Some(value.to_string()),
                    "my_feed" => my_feed = Some(value.parse().unwrap_or_default()),
                    "page" => page = Some(value.parse().unwrap_or_default()),
                    "amount" => amount = Some(value.parse().unwrap_or_default()),
                    "favourites" => favourites = Some(value.parse().unwrap_or_default()),
                    _ => {}
                }
            }
        }

        Self {
            tag,
            my_feed,
            page,
            amount,
            favourites,
        }
    }
}
