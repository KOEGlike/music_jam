use std::ops::Deref;

use super::host_page::get_jam;
use crate::components::{user::Search, Player, SongList, SongListAction, UsersBar};
use crate::model::{self, *};
use crate::pages::host_page::get_initial_update;
use codee::binary::MsgpackSerdeCodec;
use gloo::storage::{LocalStorage, Storage};
use itertools::Itertools;
use leptos::{logging::*, prelude::*};
use leptos_meta::Title;
use leptos_router::{hooks::*, *};
use leptos_use::{core::ConnectionReadyState, use_websocket, UseWebSocketReturn};

#[component]
pub fn UserPage() -> impl IntoView {
    let jam_id = move || use_params_map().with(|params| params.get("id"));
    let jam_id = Signal::derive(jam_id);
    let (jam_id_new, set_jam_id) = signal(String::new());
    Effect::new(move |_| {
        let jam_id = jam_id.get();
        if let Some(jam_id) = jam_id {
            set_jam_id.set(jam_id);
        }
    });
    Effect::new(move |_| log!("jam_id:{:?}", jam_id.get()));
    let jam_id = jam_id_new;

    let jam = Action::new(move |_: &()| async move {
        let jam_id = jam_id.get_untracked();
        get_jam(jam_id).await
    });

    Effect::new(move |_| jam.dispatch(()));
    Effect::new(move |_| {
        if let Some(jam_val) = jam.value().get() {
            if jam_val.is_err() {
                jam.dispatch(());
            }
        }
    });

    let (user_id, set_user_id) = signal(String::new());
    Effect::new(move |_| {
        let navigator = use_navigate();
        if jam_id.with(String::is_empty) {
            navigator("/", NavigateOptions::default());
            return;
        }
        let user_id: String = LocalStorage::get(jam_id.get()).unwrap_or_default();
        if user_id.is_empty() {
            navigator("/", NavigateOptions::default());
            return;
        }
        set_user_id.set(user_id);
    });

    let initial_update = LocalResource::new(move || {
        let user_id = user_id.get();
        async move { get_initial_update(user_id).await }
    });

    let (search_result, set_search_result) = signal(None);
    let (songs, set_songs) = signal(None);
    let (votes, set_votes) = signal(model::Votes::new());
    let (users, set_users) = signal(None);
    let (position, set_position) = signal(0.0);
    let (current_song, set_current_song) = signal(None);
    let (ready_state, set_ready_state) = signal(ConnectionReadyState::Connecting);

    let (send_request, set_send_request) = signal(Callback::new(|_: real_time::Request| {
        warn!("wanted to send a message to ws, but the ws is not ready yet");
    }));
    let (close, set_close) = signal(Callback::new(|_: ()| {
        warn!("wanted to close ws, but the ws is not ready yet");
    }));

    let search = move |query_id: (String, String)| {
        let request = real_time::Request::Search {
            query: query_id.0,
            id: query_id.1,
        };
        send_request.get_untracked().run(request);
    };
    let search = Callback::new(search);

    let add_song = move |song_id: String| {
        let your_song_count = songs
            .get()
            .as_ref()
            .map(|songs: &Vec<Song>| {
                songs
                    .iter()
                    .filter(|song| {
                        if let Some(id) = &song.user_id {
                            user_id.with_untracked(|user_id| *id == *user_id)
                        } else {
                            false
                        }
                    })
                    .count()
            })
            .unwrap_or(0);

        if jam
            .value()
            .get()
            .map(|jam| jam.map(|jam| jam.max_song_count))
            .unwrap_or(Ok(0))
            .unwrap_or_default()
            > your_song_count as u8
        {
            let request = real_time::Request::AddSong { song_id };
            send_request.get_untracked().run(request);
        } else {
            warn!("You have reached the maximum song count");
        }
    };
    let add_song = Callback::new(add_song);

    let add_vote = move |song_id: String| {
        log!("Adding vote for song: {}", song_id);
        let request = real_time::Request::AddVote { song_id };
        send_request.get_untracked().run(request);
    };
    let add_vote = Callback::new(add_vote);

    let remove_vote = move |song_id: String| {
        log!("Removing vote for song: {}", song_id);
        let request = real_time::Request::RemoveVote { song_id };
        send_request.get_untracked().run(request);
    };
    let remove_vote = Callback::new(remove_vote);

    let remove_song = move |song_id: String| {
        let request = real_time::Request::RemoveSong { song_id };
        send_request.get_untracked().run(request);
    };
    let remove_song = Callback::new(remove_song);

    let leave = move || {
        let request = real_time::Request::KickUser {
            user_id: user_id.get_untracked(),
        };
        send_request.get_untracked().run(request);
    };

    let delete_user_id_from_local_storage = move |_: ()| {
        LocalStorage::delete(jam_id.get_untracked());
    };
    let delete_user_id_from_local_storage = Callback::new(delete_user_id_from_local_storage);

    Effect::new(move |_| {
        if user_id.with(String::is_empty) || jam_id.with(String::is_empty) {
            return;
        }

        let UseWebSocketReturn {
            ready_state,
            message,
            close: close_ws,
            send,
            ..
        } = use_websocket::<real_time::Request, real_time::Update, MsgpackSerdeCodec>(&format!(
            "/socket?id={}",
            user_id.get_untracked()
        ));

        Effect::new(move |_| {
            log!("ready_state: {:?}", ready_state.get());
            set_ready_state.set(ready_state.get());
        });

        let send_request = move |request: real_time::Request| {
            send(&request);
        };
        let send_request = Callback::new(send_request);
        set_send_request.set(send_request);

        let close_ws = Callback::new(move |_: ()| close_ws());

        Effect::new(move |_| {
            if let Some(update) = message.get().or_else(move || {
                match initial_update.get().map(|r| r.deref().clone()) {
                    Some(Ok(update)) => Some(update),
                    Some(Err(e)) => {
                        error!("Error getting initial update: {:#?}", e);
                        None
                    }
                    None => None,
                }
            }) {
                if let Some(result) = update.search {
                    //log!("Got search result: {:#?}", result);
                    set_search_result.set(Some(result));
                }
                if let Some(songs) = update.songs {
                    set_songs.set(Some(songs));
                }
                if let Some(votes) = update.votes {
                    set_votes.set(votes);
                }
                if let Some(users) = update.users {
                    if !users
                        .iter()
                        .map(|user| &user.id)
                        .contains(&user_id.get_untracked())
                    {
                        close_ws.run(());
                        jam_id.with_untracked(|jam_id| {
                            if LocalStorage::set(jam_id, "kicked").is_err() {
                                error!("Failed to set local storage to kicked");
                            }
                        });
                        let navigator = use_navigate();
                        navigator("/", NavigateOptions::default());
                    } else {
                        set_users.set(Some(users));
                    }
                }
                if let Some(percentage) = update.position {
                    set_position.set(percentage);
                }
                if let Some(song) = update.current_song {
                    set_current_song.set(song);
                }
                if update.ended.is_some() {
                    close_ws.run(());
                    delete_user_id_from_local_storage.run(());
                    let navigator = use_navigate();
                    navigator("/", NavigateOptions::default());
                }
                if !update.errors.is_empty() {
                    error!("Errors: {:#?}", update.errors);
                }
            }
        });

        let leave = move |_: ()| {
            leave();
            close_ws.run(());
            delete_user_id_from_local_storage.run(());
            let navigator = use_navigate();
            navigator("/", NavigateOptions::default());
        };
        let leave = Callback::new(leave);
        set_close.set(leave);
    });

    let close = Callback::new(move |_| {
        close.get().run(());
    });

    view! {
        <Title text=move || {
            jam.value()
                .get()
                .map(|jam| jam.map(|jam| jam.name.clone()))
                .unwrap_or(Ok(String::from("User")))
                .unwrap_or_default()
        } />
        <div class="user-page">
            <UsersBar users close />
            <div class="center">
                <Search
                    search_result
                    search
                    add_song
                    loaded=Signal::derive(move || ready_state.get() == ConnectionReadyState::Open)
                />
                <SongList
                    songs
                    votes
                    song_list_action=SongListAction::Vote {
                        add_vote,
                        remove_vote,
                        remove_song,
                    }

                    max_song_count=Signal::derive(move || {
                        jam.value()
                            .get()
                            .map(|jam| jam.map(|jam| jam.max_song_count))
                            .unwrap_or(Ok(0))
                            .unwrap_or_default()
                    })
                />

                <Player position current_song />
            </div>
        </div>
    }
}
