use super::UserPreview;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq)]
pub struct Article {
    pub slug: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    pub description: String,
    pub created_at: String,
    pub favorites_count: i64,
    pub tag_list: Vec<String>,
    pub author: UserPreview,
    pub fav: bool,
    pub comments_count: i64,
}

impl Article {
    #[tracing::instrument]
    #[cfg(feature = "server")]
    pub async fn for_home_page(
        page: i64,
        amount: i64,
        tag: String,
        my_feed: bool,
        request_parts: axum::http::request::Parts,
    ) -> Result<Vec<Self>, sqlx::Error> {
        // let username: Option<String> = None;
        let username = crate::auth::get_username(request_parts);
        let offset = page * amount;
        sqlx::query!(
            "
SELECT
    a.slug,
    a.title,
    a.description,
    a.created_at,
    (SELECT COUNT(*) FROM FavArticles WHERE article=a.slug) as favorites_count,
    (SELECT COUNT(*) FROM comments WHERE article=a.slug) as comments_count,
    u.username, u.image,
    EXISTS(SELECT 1 FROM FavArticles WHERE article=a.slug and username=$5) as fav,
    EXISTS(SELECT 1 FROM Follows WHERE follower=$5 and influencer=u.username) as following,
    (SELECT string_agg(tag, ' ') FROM ArticleTags WHERE article = a.slug) as tag_list
FROM Articles as a
    JOIN Users as u ON a.author = u.username
WHERE
    CASE WHEN $3!='' THEN a.slug in (SELECT distinct article FROM ArticleTags WHERE tag=$3)
    ELSE 1=1
    END
    AND
    CASE WHEN $4 THEN u.username in (SELECT influencer FROM Follows WHERE follower=$5)
    ELSE 1=1
    END
ORDER BY a.created_at desc
LIMIT $1 OFFSET $2",
            amount,
            offset,
            tag,
            my_feed,
            username,
        )
        .map(|x| Self {
            slug: x.slug,
            title: x.title,
            body: None, // no need
            fav: x.fav != 0,
            // fav: x.fav.unwrap_or_default(),
            description: x.description,
            created_at: x.created_at.format(super::DATE_FORMAT).to_string(),
            favorites_count: x.favorites_count,
            // favorites_count: x.favorites_count.unwrap_or_default(),
            author: UserPreview {
                username: x.username,
                image: x.image,
                following: x.following != 0,
                // following: x.following.unwrap_or_default(),
            },
            tag_list: x
                .tag_list
                .unwrap_or_default()
                .split(' ')
                .map(ToString::to_string)
                .collect::<Vec<String>>(),
            comments_count: x.comments_count,
            // comments_count: x.comments_count.unwrap_or_default(),
        })
        .fetch_all(crate::database::server::get_db())
        .await
    }

    #[tracing::instrument(level = tracing::Level::TRACE)]
    #[cfg(feature = "server")]
    pub async fn for_user_profile_home(
        username: String,
        favourites: bool,
        page: i64,
        amount: i64,
        request_parts: axum::http::request::Parts,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let logged_user = crate::auth::get_username(request_parts);
        let offset = page * amount;
        sqlx::query!(
                "
    SELECT
        a.slug,
        a.title,
        a.description,
        a.created_at,
        u.username,
        u.image,
        (SELECT COUNT(*) FROM FavArticles WHERE article=a.slug) as favorites_count,
        (SELECT COUNT(*) FROM comments WHERE article=a.slug) as comments_count,
        EXISTS(SELECT 1 FROM FavArticles WHERE article=a.slug and username=$2) as fav,
        EXISTS(SELECT 1 FROM Follows WHERE follower=$2 and influencer=a.author) as following,
        (SELECT string_agg(tag, ' ') FROM ArticleTags WHERE article = a.slug) as tag_list
    FROM Articles as a
        JOIN Users as u ON u.username = a.author
    WHERE
        CASE WHEN $3 THEN
            EXISTS(SELECT fa.article, fa.username FROM FavArticles as fa WHERE fa.article=a.slug AND fa.username=$1)
        ELSE a.author = $1
        END
        ORDER BY a.created_at desc
        LIMIT $4 OFFSET $5",
                username,
                logged_user,
                favourites,
                amount,
                offset,
            )
            .map(|x| Self {
                slug: x.slug,
                title: x.title,
                body: None, // no need
                fav: x.fav != 0,
                description: x.description,
                created_at: x.created_at.format(super::DATE_FORMAT).to_string(),
                favorites_count: x.favorites_count,
                tag_list: x
                    .tag_list
                    .map(|x| x.split(' ').map(ToString::to_string).collect::<Vec<_>>())
                    .unwrap_or_default(),
                author: UserPreview {
                    username: x.username,
                    image: x.image,
                    following: x.following !=0,
                },
                comments_count: x.comments_count,
            })
            .fetch_all(crate::database::server::get_db())
            .await
    }

    #[cfg(feature = "server")]
    pub async fn for_article(
        slug: String,
        request_parts: axum::http::request::Parts,
    ) -> Result<Self, sqlx::Error> {
        let username = crate::auth::get_username(request_parts);
        sqlx::query!(
                r#"
        SELECT
            a.slug as slug,
            a.title as title,
            a.body as body,
            a.description as description,
            a.created_at as created_at,
            (SELECT string_agg(tag, ' ') FROM ArticleTags WHERE article = a.slug) as "tag_list: Option<String>",
            (SELECT COUNT(*) FROM FavArticles WHERE article = a.slug) as "fav_count: Option<i64>",
            (SELECT COUNT(*) FROM comments WHERE article = a.slug) as "comments_count: Option<i64>",
            u.username as username,
            u.image as image,
            EXISTS(SELECT 1 FROM FavArticles WHERE article=a.slug and username=$2) as "fav: Option<i64>",
            EXISTS(SELECT 1 FROM Follows WHERE follower=$2 and influencer=a.author) as "following: Option<i64>"
        FROM Articles a
            JOIN Users u ON a.author = u.username
        WHERE slug = $1
        "#,
                slug,
                username,
            )
            .map(|x| Self {
                slug: x.slug,
                title: x.title,
                description: x.description,
                body: Some(x.body),
                tag_list: x
                    .tag_list
                    .flatten()
                    .as_deref()
                    .unwrap_or_default()
                    .split_ascii_whitespace()
                    .map(str::to_string)
                    .collect::<Vec<_>>(),
                favorites_count: x.fav_count.flatten().unwrap_or_default(),
                created_at: x.created_at.format(super::DATE_FORMAT).to_string(),
                fav: x.fav.flatten().unwrap_or_default() != 0,
                author: UserPreview {
                    username: x.username,
                    image: x.image,
                    following: x.following.flatten().unwrap_or_default() != 0,
                },
                comments_count: x.comments_count.flatten().unwrap_or_default(),
            })
            .fetch_one(crate::database::server::get_db())
            .await
    }

    #[cfg(feature = "server")]
    pub async fn delete(
        slug: String,
        author: String,
    ) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
        sqlx::query!(
            "DELETE FROM Articles WHERE slug=$1 and author=$2",
            slug,
            author
        )
        .execute(crate::database::server::get_db())
        .await
    }
}
