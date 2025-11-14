use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq)]
pub struct MatchedArticles {
    pub slug: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub body: Option<String>,
}

impl MatchedArticles {
    #[tracing::instrument]
    #[cfg(feature = "server")]
    pub async fn search_articles(
        query: String,
        page: i64,
        amount: i64,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let offset = page * amount;
        sqlx::query!(
            // MatchedArticles,
            r#"
SELECT distinct
a.slug as slug,
snippet(articles_fts,1, '<span class="bg-yellow-300">','</span>','<span class="bg-yellow-300">  ...  </span>',10) as "title: String",
snippet(articles_fts,2, '<span class="bg-yellow-300">','</span>','<span class="bg-yellow-300">  ...  </span>',20) as "description: String",
snippet(articles_fts,3, '<span class="bg-yellow-300">','</span>','<span class="bg-yellow-300">  ...  </span>',20) as "body: String"
FROM Articles_fts AS AFTS
JOIN  Articles AS A  ON A.oid = AFTS.rowid
WHERE Articles_fts MATCH $3
order by rank
LIMIT $1 OFFSET $2"#,
            amount,
            offset,
            query,
        )
        .map(|x| Self {
            slug: x.slug,
            title: x.title,
            description: x.description,
            body: x.body,
        })
        .fetch_all(crate::database::server::get_db())
        .await
    }
}
