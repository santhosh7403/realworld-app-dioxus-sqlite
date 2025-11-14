#[cfg(feature = "server")]
pub mod server {
    static DB: std::sync::OnceLock<sqlx::SqlitePool> = std::sync::OnceLock::new();

    async fn create_pool() -> sqlx::SqlitePool {
        let database_url =
            std::env::var("DATABASE_URL").expect("no database url found in ENV vars");
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(8)
            .connect(database_url.as_str())
            .await
            .expect("could not connect to database");

        pool
    }

    pub async fn init_db() -> Result<(), ()> {
        DB.set(create_pool().await).map_err(|_| ())
    }

    pub fn get_db() -> &'static sqlx::SqlitePool {
        DB.get().expect("database initializing had issues")
    }
}
