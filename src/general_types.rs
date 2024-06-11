use serde::{Deserialize, Serialize};
use std::error::Error;
#[cfg(feature ="ssr")]
use axum::extract::FromRef;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SpotifyCredentials {
    pub id: String,
    pub secret: String
}

#[cfg(feature ="ssr")]
#[derive(FromRef,Clone, Debug)]
pub struct AppState {
    pub db: Db,
    pub reqwest_client: reqwest::Client,
    pub spotify_credentials: SpotifyCredentials,
    pub leptos_options: leptos::LeptosOptions,
}

#[cfg(feature ="ssr")]
#[derive(Clone, Debug)]
pub struct Db{
    pub pool: sqlx::PgPool,
    pub url: String
}

#[cfg(feature ="ssr")]
impl AppState {
    pub async fn new(leptos_options: leptos::LeptosOptions) -> Result<Self, Box<dyn Error>> {
        dotenvy::dotenv().unwrap();
        let reqwest_client = reqwest::Client::new();
        let spotify_id = std::env::var("SPOTIFY_ID")?;
        let spotify_secret = std::env::var("SPOTIFY_SECRET")?;
        let db_url = std::env::var("DATABASE_URL")?;
        let db_pool = sqlx::PgPool::connect(&db_url).await?;

        let spotify_credentials=SpotifyCredentials{
            id:spotify_id,
            secret:spotify_secret
        };

        Ok(Self {
            db:Db{
                pool:db_pool,
                url:db_url
            },
            reqwest_client,
            spotify_credentials,
            leptos_options
        })
    }
}

pub type JamId = String;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub id: String,
    pub jam_id: String,
    pub name: String,
    pub pfp_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Jam {
    pub id: String,
    pub name: String,
    pub max_song_count: i8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Song {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub artist: String,
    pub album: String,
    pub duration: i32
}

