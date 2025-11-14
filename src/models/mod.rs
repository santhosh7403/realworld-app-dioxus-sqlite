mod user;
pub use user::{User, UserPreview};
mod pagination;
pub use pagination::Pagination;
pub mod article;

pub use article::Article;

mod comment;
pub use comment::Comment;

const DATE_FORMAT: &str = "%d/%m/%Y %H:%M";
mod search;
pub use search::MatchedArticles;
