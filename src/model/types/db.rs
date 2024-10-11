use super::Error;

#[derive(Clone, Debug)]
pub struct Db {
    pub pool: sqlx::PgPool,
    pub url: String,
}

impl Db {
    pub async fn new(url: String) -> Result<Self, Error> {
        println!("creating db pool with url: {} .......", url);
        let pool = sqlx::postgres::PgPoolOptions::new()
            .idle_timeout(Some(std::time::Duration::from_secs(60 * 15)))
            .acquire_timeout(std::time::Duration::from_secs(60 * 5))
            .max_connections(15)
            .min_connections(5)
            .max_lifetime(Some(std::time::Duration::from_secs(60 * 60 * 24)))
            .acquire_timeout(std::time::Duration::from_secs(60 * 5))
            .connect(&url)
            .await?;

        println!("running migrations...");
        sqlx::migrate!("db/migrations")
            .run(&pool)
            .await
            .map_err(|e| Error::Database(format!("The migration failed: {}", e)))?;
        println!("migrations ran...");

        Ok(Db { pool, url })
    }
}
