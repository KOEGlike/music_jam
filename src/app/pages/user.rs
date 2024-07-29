use crate::app::components::{Search, SongList, SongListAction};
use crate::general;
use gloo::storage::{LocalStorage, Storage};
use leptos::{logging::*, prelude::*, *};
use leptos_router::*;
use leptos_use::{use_websocket, UseWebsocketReturn};

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

    let (send_request, set_send_request) =
        create_signal(Callback::new(|_: general::real_time::Request| {
            warn!("wanted to send a message to ws, but the ws is not ready yet");
        }));
    let (close, set_close) = create_signal(Callback::new(|_: ()| {
        warn!("wanted to close ws, but the ws is not ready yet");
    }));

    let search = move |query: String| {
        let request = general::real_time::Request::Search { query };
        send_request()(request);
    };
    let search = Callback::new(search);

    let add_song = move |song_id: String| {
        let request = general::real_time::Request::AddSong { song_id };
        send_request()(request);
    };
    let add_song = Callback::new(add_song);

    let add_vote = move |song_id: String| {
        log!("Adding vote for song: {}", song_id);
        let request = general::real_time::Request::AddVote { song_id };
        send_request()(request);
    };
    let add_vote = Callback::new(add_vote);

    let remove_vote = move |song_id: String| {
        log!("Removing vote for song: {}", song_id);
        let request = general::real_time::Request::RemoveVote { song_id };
        send_request()(request);
    };
    let remove_vote = Callback::new(remove_vote);

    let request_update = move || {
        let request = general::real_time::Request::Update;
        send_request()(request);
        log!("Sent update request");
    };

    create_effect(move |_| {
        if user_id.with(String::is_empty) || jam_id.with(String::is_empty) {
            return;
        }

        let UseWebsocketReturn {
            ready_state,
            message_bytes,
            close:close_ws,
            send_bytes,
            ..
        } = use_websocket(&format!("/socket?id={}", user_id.get_untracked()));

        let send_request = move |request: general::real_time::Request| {
            let bin = rmp_serde::to_vec(&request).unwrap();
            send_bytes(bin);
        };
        let send_request = Callback::new(send_request);
        set_send_request(send_request);

        let close_ws = Callback::new(move|_:()|close_ws());

        let update = move || {
            use general::real_time;
            let bin = match message_bytes() {
                Some(bin) => bin,
                None => return None,
            };
            let update = match rmp_serde::from_slice::<real_time::Update>(&bin) {
                Ok(update) => update,
                Err(e) => real_time::Update::Error(general::Error::Decode(format!(
                    "Error deserializing update: {:?}",
                    e
                ))),
            };
            Some(update)
        };

        create_effect(move |_| {
            use general::real_time;
            if let Some(update) = update() {
                match update {
                    real_time::Update::Error(e) => error!("Error: {:#?}", e),
                    real_time::Update::Search(result) => set_search_result(Some(result)),
                    real_time::Update::Songs(songs) => set_songs(Some(songs)),
                    real_time::Update::Votes(votes) => set_votes(votes),
                    real_time::Update::Users(users) => set_users(Some(users)),
                    real_time::Update::Position{percentage} => set_position(percentage),
                    real_time::Update::CurrentSong(song) => set_current_song(song),
                    real_time::Update::Ended => {
                        close_ws(());
                        let navigator = use_navigate();
                        navigator("/", NavigateOptions::default());
                    }
                }
            }
        });

        let message_is_null = create_memo(move |_| message_bytes.with(Option::is_none));
        create_effect(move |_| {
            if message_is_null() {
                request_update();
            } else {
                set_search_result(Some(vec![]));
            };
        });

        let delete_user = create_action(move |_: &()| {
            async move {
                let id = user_id.get_untracked();
                delete_user(id).await
            }
        });
        let close = Callback::new(move |_: ()| {
            delete_user.dispatch(());
            close_ws(());
        });
        set_close(close);
    });

    view! {
        <Search search_result=search_result search=search add_song=add_song/>
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
    use crate::general::{AppState, kick_user};
    let app_state=expect_context::<AppState>();
    kick_user(&id, &app_state.db.pool).await?;
    Ok(())
}