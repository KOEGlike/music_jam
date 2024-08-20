use crate::app::components::{Search, SongList, SongListAction};
use codee::string::JsonSerdeWasmCodec;
use crate::general::{self, *};
use codee::binary::MsgpackSerdeCodec;
use gloo::storage::{LocalStorage, Storage};
use leptos::{logging::*, prelude::*, *};
use leptos_router::*;
use leptos_use::core::ConnectionReadyState;
use leptos_use::{use_websocket, UseWebSocketReturn};

#[component]
pub fn UserPage() -> impl IntoView {
    let params = use_params_map();
    let jam_id = move || params.with(|params| params.get("id").cloned());
    let jam_id = Signal::derive(move || jam_id().unwrap_or_default());
    let (user_id, set_user_id) = create_signal(String::new());
    create_effect(move |_| {
        let navigator = use_navigate();
        if jam_id.with(String::is_empty) {
            navigator("/", NavigateOptions::default());
            return;
        }
        let user_id: String = LocalStorage::get(jam_id()).unwrap_or_default();
        if user_id.is_empty() {
            navigator("/", NavigateOptions::default());
            return;
        }
        set_user_id(user_id);
    });

    let (search_result, set_search_result) = create_signal(None);
    let (songs, set_songs) = create_signal(None);
    let (votes, set_votes) = create_signal(general::Votes::new());
    let (users, set_users) = create_signal(None);
    let (position, set_position) = create_signal(0.0);
    let (current_song, set_current_song) = create_signal(None);

    let (send_request, set_send_request) = create_signal(Callback::new(|_: real_time::Request| {
        warn!("wanted to send a message to ws, but the ws is not ready yet");
    }));
    let (close, set_close) = create_signal(Callback::new(|_: ()| {
        warn!("wanted to close ws, but the ws is not ready yet");
    }));

    let search = move |query: String| {
        let request = real_time::Request::Search { query };
        send_request()(request);
    };
    let search = Callback::new(search);

    let add_song = move |song_id: String| {
        let request = real_time::Request::AddSong { song_id };
        send_request()(request);
    };
    let add_song = Callback::new(add_song);

    let add_vote = move |song_id: String| {
        log!("Adding vote for song: {}", song_id);
        let request = real_time::Request::AddVote { song_id };
        send_request()(request);
    };
    let add_vote = Callback::new(add_vote);

    let remove_vote = move |song_id: String| {
        log!("Removing vote for song: {}", song_id);
        let request = real_time::Request::RemoveVote { song_id };
        send_request()(request);
    };
    let remove_vote = Callback::new(remove_vote);

    let request_update = move || {
        let request = real_time::Request::Update;
        send_request()(request);
        log!("Sent update request");
    };

    create_effect(move |_| {
        if user_id.with(String::is_empty) || jam_id.with(String::is_empty) {
            return;
        }

        let UseWebSocketReturn {
            ready_state,
            message,
            close: close_ws,
            send,
            ..
        } = use_websocket::<real_time::Message, JsonSerdeWasmCodec>(&format!(
            "/socket?id={}",
            user_id.get_untracked()
        ));

        let send_request = move |request: real_time::Request| {
            send(&real_time::Message::Request(request));
        };
        let send_request = Callback::new(send_request);
        set_send_request(send_request);

        let close_ws = Callback::new(move |_: ()| close_ws());

        create_effect(move |_| {
            if let ConnectionReadyState::Open = ready_state() {
                let request = real_time::Request::Update;
                send_request(request);
                set_search_result(Some(vec![]));
            }
        });

        create_effect(move |_| {
            log!("Update: {:#?}", message());
            if let Some(real_time::Message::Update(update)) = message() {
                
                if let Some(result) = update.search {
                    set_search_result(Some(result));
                }
                if let Some(songs) = update.songs {
                    set_songs(Some(songs));
                }
                if let Some(votes) = update.votes {
                    set_votes(votes);
                }
                if let Some(users) = update.users {
                    set_users(Some(users));
                }
                if let Some(percentage) = update.position {
                    set_position(percentage);
                }
                if let Some(song) = update.current_song {
                    set_current_song(song);
                }
                if update.ended.is_some() {
                    close_ws(());
                    let navigator = use_navigate();
                    navigator("/", NavigateOptions::default());
                }
                if !update.errors.is_empty() {
                    error!("Errors: {:#?}", update.errors);
                }
            }
        });

        let delete_user = create_action(move |_: &()| async move {
            let id = user_id.get_untracked();
            delete_user(id).await
        });
        let close = Callback::new(move |_: ()| {
            delete_user.dispatch(());
            close_ws(());
        });
        set_close(close);
    });

    view! {
        <Search search_result search add_song=add_song/>
        <SongList
            songs=songs
            votes=votes
            request_update=request_update
            song_list_action=SongListAction::Vote {
                add_vote,
                remove_vote,
            }
        />
    }
}

#[server]
async fn delete_user(id: String) -> Result<(), ServerFnError> {
    use crate::general::{kick_user, AppState};
    let app_state = expect_context::<AppState>();
    kick_user(&id, &app_state.db.pool).await?;
    Ok(())
}
