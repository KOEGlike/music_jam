pub mod fileserv;
pub use fileserv::*;

use crate::general_types::AppState;
use axum::{routing::get, Router};

use leptos::*;
use leptos_axum::*;
use leptos_router::RouteListing;

use crate::app::App;

pub fn new(leptos_routes: Vec<RouteListing>, app_state: AppState) -> Router {
    Router::new()
        .leptos_routes_with_context(
            &app_state,
            leptos_routes,
            {
                let state = app_state.clone();
                move || provide_context(state.clone())
            },
            App,
        )
        .route("/socket", get(crate::app::socket::socket))
        .fallback(file_and_error_handler)
        .with_state(app_state.clone())
}

#[component]
fn TestSocketRead() -> impl IntoView {
    use crate::general_types::*;
    use leptos_use::{core::ConnectionReadyState, use_websocket, UseWebsocketReturn};
    let id = "";

    let UseWebsocketReturn {
        ready_state,
        message_bytes,
        send_bytes,
        open,
        ..
    } = use_websocket(&format!("wss://localhost:3000/socket?id={}", id));

    let update = move || match message_bytes() {
        Some(m) => rmp_serde::from_slice::<real_time::Update>(&m).unwrap(),
        None => real_time::Update::Error(real_time::Error::Database(
            "idk this is on the client side so it should never happen".to_string(),
        )),
    };

    let send = move || {
        let message = real_time::Request::AddSong { song_id: "song_id".to_string() };
        let message = rmp_serde::to_vec(&message).unwrap();
        send_bytes(message);
    };

    view! {
        <button on:click= move |_|open() >"Open"</button>
        <br/>
        <button on:click= move |_|send() >"Send"</button>
        <br/>
        {move || match ready_state() {
            ConnectionReadyState::Connecting => "Connecting",
            ConnectionReadyState::Open => "Open",
            ConnectionReadyState::Closing => "Closing",
            ConnectionReadyState::Closed => "Closed",
        }}
        <br/>
        {move || format!("{:#?}", update())}


    }
}
