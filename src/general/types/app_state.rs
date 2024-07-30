use std::error::Error as StdError;
use crate::general::types::*;
use axum::extract::FromRef;

#[derive(FromRef, Clone, Debug)]
pub struct AppState {
    pub db: Db,
    pub reqwest_client: reqwest::Client,
    pub spotify_credentials: SpotifyCredentials,
    pub leptos_options: leptos::LeptosOptions,
}

impl AppState {
    pub async fn new(leptos_options: leptos::LeptosOptions) -> Result<Self, Box<dyn StdError>> {
        dotenvy::dotenv()?;
        let reqwest_client = reqwest::Client::new();
        let spotify_id = std::env::var("SPOTIFY_ID")?;
        let spotify_secret = std::env::var("SPOTIFY_SECRET")?;
        let db_url = std::env::var("DATABASE_URL")?;
        let db_pool = sqlx::postgres::PgPoolOptions::new()
            .idle_timeout(Some(std::time::Duration::from_secs(60 * 15)))
            .acquire_timeout(std::time::Duration::from_secs(60 * 5))
            .max_connections(15)
            .min_connections(5)
            .max_lifetime(Some(std::time::Duration::from_secs(60 * 60 * 24)))
            .acquire_timeout(std::time::Duration::from_secs(60 * 5))
            .connect(&db_url)
            .await?;

        let spotify_credentials = SpotifyCredentials {
            id: spotify_id,
            secret: spotify_secret,
        };

        Ok(Self {
            db: Db {
                pool: db_pool,
                url: db_url,
            },
            reqwest_client,
            spotify_credentials,
            leptos_options,
        })
    }
}