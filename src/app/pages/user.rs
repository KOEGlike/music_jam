use crate::app::components::{Search, SongAction, SongList};
use crate::app::general;
use gloo::{storage::*, timers::callback::Interval};
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
        if jam_id().is_empty() {
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

    let loaded = {
        move || {
            let UseWebsocketReturn {
                ready_state,
                message_bytes,
                open,
                close,
                send_bytes,
                ..
            } = use_websocket(&format!("/socket?id={}", user_id.get_untracked()));

            let send_bytes = Callback::new(send_bytes);
            let (search_result, set_search_result) = create_signal(None);
            let (songs, set_songs) = create_signal(None);
            let (votes, set_votes) = create_signal(general::Votes::new());
            let (users, set_users) = create_signal(None);
            let (your_votes, set_your_votes) = create_signal(vec![]);

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
                        real_time::Update::YourVotes(votes) => set_your_votes(votes),
                    }
                }
            });

            let search = move |query: String| {
                let request = general::real_time::Request::Search { query };
                let bin = rmp_serde::to_vec(&request).unwrap();
                send_bytes(bin);
            };
            
            let add_song = move |song_id: String| {
                let request = general::real_time::Request::AddSong { song_id };
                let bin = rmp_serde::to_vec(&request).unwrap();
                send_bytes(bin);
            };

            let vote = move |song_id: String| {
                let request = general::real_time::Request::AddVote { song_id };
                let bin = rmp_serde::to_vec(&request).unwrap();
                send_bytes(bin);
            };
            let vote = Callback::new(vote);

            let request_update = move || {
                let request = general::real_time::Request::Update;
                let bin = rmp_serde::to_vec(&request).unwrap();
                send_bytes(bin);
                log!("Sent update request");
            };
            let request_update_interval = Interval::new(60 * 1000, request_update);
            request_update_interval.forget();

            let message_is_null = create_memo(move |_| message_bytes.with(Option::is_none));
            create_effect(move |_| {
                if message_is_null() {
                    request_update();
                } else {
                    set_search_result(Some(vec![]));
                };
            });

            view! {
                <Search search_result=search_result search=search add_song=add_song/>
                <SongList
                    songs=songs
                    votes=votes
                    request_update=request_update
                    song_action=SongAction::Vote(vote)
                />
            }
        }
    };

    view! {
        <Show
            when=move || user_id.with(move |user_id| !user_id.is_empty())
            fallback=move || "Loading..."
        >
            {loaded}
        </Show>
    }
}
