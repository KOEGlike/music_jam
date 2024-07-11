use leptos::{
    logging::{error, log},
    prelude::*,
    *,
};
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
    let (player_is_connected, set_player_is_connected) = create_signal(false);
    let token = create_action(move |_: &()| {
        let host_id = host_id.clone();
        async move { get_access_token(host_id).await }
    });

    token.dispatch(());

    let connect = create_action(move |_: &()| async move { sp::connect().await });

    create_effect(move |_| match connect.value().get() {
        Some(Ok(_)) => {
            set_player_is_connected(true);
        }
        Some(Err(e)) => {
            error!("error while connecting to spotify:{:?}", e);
        }
        None => {}
    });

    

    create_effect(move |_|{
        if !sp::player_ready()   {
            if let Some(Ok(token_value)) = token.value().get() {
                sp::init(
                    move || {
                        token.dispatch(());
                        token_value.access_token.clone()
                    },
                    move || {
                        log!("player is ready");
                        connect.dispatch(());
                    },
                    "jam",
                    1.0,
                    false,
                );
            }
        }
    });

    let toggle_play = create_action(move |_: &()| async {
        if let Err(e) = sp::toggle_play().await {
            error!("Error toggling play: {:?}", e);
        }
    });

    view! {
        {move || {
            if player_is_connected() {
                view! { <button on:click=move |_| toggle_play.dispatch(())>"Play"</button> }
                    .into_view()
            } else {
                "loading...".into_view()
            }
        }}
    }
}
