use crate::app::components::{host_only::Player, Share, SongList, SongListAction, UsersBar};
use crate::app::general::types::*;
use gloo::storage::{LocalStorage, Storage};
use leptos::{logging::*, prelude::*, *};
use leptos_router::{use_navigate, NavigateOptions};
use leptos_use::{use_websocket, UseWebsocketReturn};

#[component]
pub fn HostPage() -> impl IntoView {
    let (host_id, set_host_id) = create_signal(String::new());
    let host_id = create_memo(move |_| host_id());
    create_effect(move |_| {
        let host_id: String = LocalStorage::get("host_id").unwrap_or_default();
        if host_id.is_empty() {
            let navigator = use_navigate();
            navigator("/", NavigateOptions::default());
        }
        set_host_id(host_id);
    });

    let (users, set_users) = create_signal(None);
    let (songs, set_songs) = create_signal(None::<Vec<Song>>);
    let (votes, set_votes) = create_signal(Votes::new());

    let (send_request, set_send_request) = create_signal(Callback::new(|_: real_time::Request| {
        warn!("wanted to send a message to ws, but the ws is not ready yet");
    }));

    let top_song_id = move || match songs() {
        Some(songs) => songs
            .iter()
            .max_by_key(|song| {
                votes()
                    .get(&song.id)
                    .copied()
                    .unwrap_or(Vote {
                        votes: 0,
                        have_you_voted: None,
                    })
                    .votes
            })
            .map(|song| song.id.clone()),
        None => None,
    };
    let top_song_id = Signal::derive(top_song_id);

    let remove_song = move |id| {
        let request = real_time::Request::RemoveSong { song_id: id };
        send_request()(request);
    };
    let remove_song = Callback::new(remove_song);

    let request_update = move || {
        let request = real_time::Request::Update;
        send_request()(request);
    };

    let kick_user = move |id| {
        let request = real_time::Request::KickUser { user_id: id };
        send_request()(request);
    };
    let kick_user = Callback::new(kick_user);

    let reset_votes = move |_: ()| {
        let request = real_time::Request::ResetVotes;
        send_request()(request);
    };
    let reset_votes = Callback::new(reset_votes);

    log!("host_id: {}", host_id.get_untracked());

    create_effect(move |_| {
        if host_id.with(String::is_empty) {
            return;
        }

        let UseWebsocketReturn {
            ready_state,
            message_bytes,
            close,
            send_bytes,
            ..
        } = use_websocket(&format!("/socket?id={}", host_id.get_untracked()));
        let send_request = move |request: real_time::Request| {
            let bin = rmp_serde::to_vec(&request).unwrap();
            send_bytes(bin);
        };
        let send_request = Callback::new(send_request);
        set_send_request(send_request);

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
                    real_time::Update::Search(_) => error!("Unexpected search update"),
                }
            }
        });
    });

    view! {
        <Player host_id=host_id.get_untracked() top_song_id=top_song_id reset_votes=reset_votes/>
        <SongList
            songs=songs
            votes=votes
            request_update=request_update
            song_list_action=SongListAction::Remove(remove_song)
        />
        <UsersBar users=users kick_user=kick_user/>
    }
}
