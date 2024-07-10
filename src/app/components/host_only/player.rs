use leptos::{logging::log, prelude::*, *};
use leptos_router::*;
use leptos_use::{use_websocket, UseWebNotificationReturn, UseWebSocketOptions};
use rust_spotify_web_playback_sdk::prelude as sp;

#[server]
async fn get_access_token(host_id: String) -> Result<rspotify::Token, ServerFnError> {
    use crate::app::general::*;
    let app_state = expect_context::<AppState>();
    let pool = &app_state.db.pool;
    let reqwest_client = &app_state.reqwest_client;

    let jam_id = check_id_type(&host_id, pool).await;
    let jam_id = match jam_id {
        Ok(id) => id,
        Err(sqlx::Error::RowNotFound) => {
            leptos_axum::redirect("/");
            return Err(ServerFnError::Request("Host not found".to_string()));
        }
        Err(e) => return Err(ServerFnError::ServerError(e.to_string())),
    };
    let jam_id = match jam_id {
        IdType::Host(id) => id.jam_id,
        IdType::User(_) => {
            leptos_axum::redirect("/");
            return Err(ServerFnError::Request(
                "the id was found, but it belongs to a user".to_string(),
            ));
        }
    };

    let token = match get_access_token(pool, &jam_id, reqwest_client).await {
        Ok(token) => token,
        Err(e) => return Err(ServerFnError::ServerError(e.into())),
    };

    Ok(token)
}

#[component]
pub fn Player(host_id: String) -> impl IntoView {
    let (player_is_ready, set_player_is_ready) = create_signal(false);
    let token = create_action(move |_: &()| {
        let host_id = host_id.clone();
        async move { get_access_token(host_id).await }
    });

    create_effect(move |_| {
        if !token.pending().get() {
            token.dispatch(());
            return;
        }

        let token_value = token.value().get();
        let token_value = match token_value {
            Some(Ok(token)) => token,
            Some(Err(e)) => {
                log!("{:?}", e);
                return;
            }
            None => return,
        };

        sp::init(
            move || {
                token.dispatch(());
                token_value.access_token.clone()
            },
            move || set_player_is_ready(true),
            "jam",
            1.0,
            false,
        );
    });
}
