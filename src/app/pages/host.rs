use crate::app::components::{host_only::Player, Share, SongAction, SongList, UsersBar};
use crate::app::general::types::*;
use gloo::storage::{LocalStorage, Storage};
use leptos::{
    logging::{error, log},
    prelude::*,
    *,
};
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

        let send_bytes = Callback::new(send_bytes);

        let (users, set_users) = create_signal(None);
        let (songs, set_songs) = create_signal(None);
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
                    real_time::Update::Users(users) => set_users(Some(users)),
                    real_time::Update::Songs(songs) => set_songs(Some(songs)),
                    real_time::Update::Votes(votes) => set_votes(votes),
                    real_time::Update::Error(e) => error!("Error: {:#?}", e),
                    real_time::Update::Search(_) => {
                        error!("Unexpected search update")
                    }
                }
            }
        });

        let remove_song = move |id| {
            let request = real_time::Request::RemoveSong { song_id: id };
            let bin = rmp_serde::to_vec(&request).unwrap();
            send_bytes(bin);
        };
        let remove_song = Callback::new(remove_song);

        let request_update = move || {
            send_bytes(rmp_serde::to_vec(&real_time::Request::Update).unwrap());
        };

        let kick_user = move |id| {
            let request = real_time::Request::KickUser { user_id: id };
            let bin = rmp_serde::to_vec(&request).unwrap();
            send_bytes(bin);
        };
        let kick_user = Callback::new(kick_user);

        let top_song = move || match songs() {
            Some(songs) => songs
                .iter()
                .max_by_key(|song| votes().get(&song.id).copied().unwrap_or(0))
                .cloned(),
            None => None,
        };
        let top_song = Signal::derive(top_song);

        let reset_votes = move || {
            let request = real_time::Request::ResetVotes;
            let bin = rmp_serde::to_vec(&request).unwrap();
            send_bytes(bin);
        };

        let view = view! {
            <UsersBar users=users kick_user=kick_user/>
            <Player host_id=host_id() top_song=top_song reset_votes=reset_votes/>
            <SongList
                songs=songs
                votes=votes
                request_update=request_update
                song_action=SongAction::Remove(remove_song)
            />
        }
        .into_view();
        Some(view)
    };

    view! {
        {move || match body() {
            Some(body) => body,
            None => "loading...".into_view(),
        }}
    }
}
