use crate::model::types::*;
use axum::extract::FromRef;

#[derive(FromRef, Clone, Debug)]
pub struct AppState {
    pub db: Db,
    pub reqwest_client: reqwest::Client,
    pub spotify_credentials: SpotifyCredentials,
    pub leptos_options: leptos::prelude::LeptosOptions,
    pub site_url: String,
}

impl AppState {
    pub async fn new(
        leptos_options: leptos::prelude::LeptosOptions,
        spotify_id: String,
        spotify_secret: String,
        db_url: String,
        site_url: String,
    ) -> Result<Self, Error> {
        let reqwest_client = reqwest::Client::new();
        println!("Connecting to database...",);

        let db = Db::new(db_url).await?;
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
            site_url,
        })
    }
}
