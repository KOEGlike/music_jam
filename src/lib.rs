pub mod app;
pub mod error_template;

#[cfg(feature = "ssr")]
pub mod fileserv;
#[cfg(feature="ssr")]
pub mod startup;


#[cfg(feature = "ssr")]
#[derive(Clone)]
struct AppState{
    pub db_pool:sqlx::PgPool,
    pub reqwest_client: reqwest::Client,
    pub spotify_id: String,
    pub spotify_secret: String,
}

#[cfg(feature = "ssr")]
impl AppState {
    pub fn new(db_pool: sqlx::PgPool) -> Self {
        dotenvy::dotenv().unwrap();
        let reqwest_client = reqwest::Client::new();
        let spotify_id = std::env::var("SPOTIFY_ID").unwrap();
        let spotify_secret = std::env::var("SPOTIFY_SECRET").unwrap();
        Self {
            db_pool,
            reqwest_client,
            spotify_id,
            spotify_secret
        }
    }
}



#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;
    console_error_panic_hook::set_once();
    leptos::mount_to_body(App);
}
