use crate::app::components::{host_only::Player};
use crate::app::general::types::*;
use gloo::storage::{LocalStorage, Storage};
use leptos::{logging::{log, error}, prelude::*, *};
use leptos_use::{use_websocket, UseWebsocketReturn};
use rspotify::model::user;

#[component]
pub fn HostPage() -> impl IntoView {
    let (host_id, set_host_id) = create_signal(String::new());

    create_effect(move |_| {
        set_host_id(LocalStorage::get("host_id").unwrap());
    });

    let body = move || {
        if host_id().is_empty() {
            return None;
        }
        log!("host_id: {}", host_id());
        let UseWebsocketReturn {
            ready_state,
            message_bytes,
            open,
            close,
            send_bytes,
            ..
        } = use_websocket(&format!("/socket?id={}", host_id()));

        let (users, set_users) = create_signal(Vec::new());
        let (songs, set_songs) = create_signal(Vec::new());
        let (votes, set_votes) = create_signal(Votes::new());

        let update = move || {
            let bin = match message_bytes() {
                Some(bin) => bin,
                None => return None,
            };
            let update = match rmp_serde::from_slice::<real_time::Update>(&bin) {
                Ok(update) => update,

                Err(e) => real_time::Update::Error(Error::Decode(format!(
                    "Error deserializing update: {:?}",
                    e
                ))),
            };
            Some(update)
        };

        create_effect(move |_| {
            if let Some(update) = update() {
                match update {
                    real_time::Update::Users(users) => set_users(users),
                    real_time::Update::Songs(songs) => set_songs(songs),
                    real_time::Update::Votes(votes) => set_votes(votes),
                    real_time::Update::Error(e) => error!("Error: {:#?}", e),
                    real_time::Update::Search(_) => {error!("Unexpected search update")},
                }
            }
        });

        let view = view! { <Player host_id=host_id()/> }.into_view();
        Some(view)
    };

    view! {
        {move || match body() {
            Some(body) => body,
            None => "loading...".into_view(),
        }}
    }
}
