#[derive(Clone, Debug)]
pub struct Db {
    pub pool: sqlx::PgPool,
    pub url: String,
}