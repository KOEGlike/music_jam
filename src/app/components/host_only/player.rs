use leptos_router::*;
use leptos::{logging::log, prelude::*, *};
use leptos_use::{use_websocket, UseWebNotificationReturn, UseWebSocketOptions};
use rust_spotify_web_playback_sdk::prelude as sp;

#[server]
async fn get_access_token(host_id: &str) -> Result<rspotify::Token, ServerFnError> {
    use crate::app::general::*;
    let app_state = expect_context::<AppState>();
    let pool= &app_state.db.pool;
    let id=check_id_type(host_id, pool).await;
    get_access_token(pool, jam_id)
    Ok(())
}

#[component]
pub fn Player(host_id: &str) -> impl IntoView {
    

    view! {
    }
}