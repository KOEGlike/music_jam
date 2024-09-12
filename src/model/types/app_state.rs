use crate::model::types::*;
use axum::extract::FromRef;
use std::error::Error as StdError;

#[derive(FromRef, Clone, Debug)]
pub struct AppState {
    pub db: Db,
    pub reqwest_client: reqwest::Client,
    pub spotify_credentials: SpotifyCredentials,
    pub leptos_options: leptos::LeptosOptions,
}

impl AppState {
    pub async fn new(leptos_options: leptos::LeptosOptions) -> Result<Self, Error> {
        println!("Loading configuration for app_state...");
        if dotenvy::dotenv().is_err(){
            eprintln!("didn't find env file")
        };
        let reqwest_client = reqwest::Client::new();
        let spotify_id = std::env::var("SPOTIFY_ID")?;
        let spotify_secret = std::env::var("SPOTIFY_SECRET")?;
        let db_url = std::env::var("DATABASE_URL")?;
        println!("Connecting to database...",);
        
        let db=Db::new(db_url).await?;
        println!("Connected to database...");

        let spotify_credentials = SpotifyCredentials {
            id: spotify_id,
            secret: spotify_secret,
        };

        Ok(Self {
            db,
            reqwest_client,
            spotify_credentials,
            leptos_options,
        })
    }
}
