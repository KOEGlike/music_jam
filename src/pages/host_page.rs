use std::ops::Deref;

use crate::components::{host::Player, Modal, Share, SongList, SongListAction, UsersBar};
use crate::model::types::*;
use codee::binary::MsgpackSerdeCodec;
use gloo::storage::{LocalStorage, Storage};
use leptos::reactive::transition;
use leptos::{logging::*, prelude::*};
use leptos_meta::Title;
use leptos_router::{
    hooks::{use_navigate, use_params_map},
    NavigateOptions,
};
use leptos_use::{use_websocket, UseWebSocketReturn};

#[component]
pub fn HostPage() -> impl IntoView {
    let (error_message, set_error_message) = signal(String::new());

    let (host_id, set_host_id) = signal(String::new());
    let host_id = Memo::new(move |_| {
        if host_id.with(String::is_empty) {
            None
        } else {
            Some(host_id.get_untracked())
        }
    });
    Effect::new(move |_| {
        let host_id: String = LocalStorage::get("host_id").unwrap_or_default();
        if host_id.is_empty() {
            let navigator = use_navigate();
            navigator("/", NavigateOptions::default());
        }
        set_host_id(host_id);
    });

    let initial_update = LocalResource::new(move || {
        let host_id = host_id.get();
        async move {
            if let Some(host_id) = host_id {
                get_initial_update(host_id).await
            } else {
                Err(ServerFnError::Request("host_id is empty".to_string()))
            }
        }
    });

    let jam_id = move || use_params_map().with(|params| params.get("id"));
    let jam_id = Signal::derive(jam_id);
    let (jam_id_new, set_jam_id) = signal(String::new());
    Effect::new(move |_| {
        let jam_id = jam_id();
        if let Some(jam_id) = jam_id {
            set_jam_id(jam_id);
        }
    });
    Effect::new(move |_| log!("jam_id:{:?}", jam_id()));
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

    let (users, set_users) = signal(None);
    let (songs, set_songs) = signal(None::<Vec<Song>>);
    let (votes, set_votes) = signal(Votes::new());

    let (send_request, set_send_request) = signal(Callback::new(|_: real_time::Request| {
        warn!("wanted to send a message to ws, but the ws is not ready yet");
    }));
    let (close, set_close) = signal(Callback::new(|_: ()| {
        warn!("wanted to close ws, but the ws is not ready yet");
    }));

    let remove_song = move |id| {
        let request = real_time::Request::RemoveSong { song_id: id };
        send_request.get_untracked().run(request);
    };
    let remove_song = Callback::new(remove_song);

    let kick_user = move |id| {
        let request = real_time::Request::KickUser { user_id: id };
        send_request.get_untracked().run(request);
    };
    let kick_user = Callback::new(kick_user);

    let next_song = move |_: ()| {
        let request = real_time::Request::NextSong;
        send_request.get_untracked().run(request);
    };
    let next_song = Callback::new(next_song);

    let set_song_position = move |percentage| {
        let request = real_time::Request::Position { percentage };
        send_request.get_untracked().run(request);
    };
    let set_song_position = Callback::new(set_song_position);

    Effect::new(move |_| log!("host_id:{:?}", host_id()));

    Effect::new(move |_| {
        let host_id = match host_id() {
            Some(host_id) => host_id,
            None => return,
        };

        let UseWebSocketReturn {
            ready_state,
            message,
            close: close_ws,
            send,
            ..
        } = use_websocket::<real_time::Request, real_time::Update, MsgpackSerdeCodec>(&format!(
            "/socket?id={}",
            host_id
        ));

        Effect::new(move |_| {
            log!("ready_state: {:?}", ready_state.get_untracked());
        });

        let send_request = Callback::new(move |request| send(&request));
        set_send_request(send_request);

        let delete_jam = Action::new(move |_: &()| {
            let host_id = host_id.clone();
            async move { delete_jam(host_id).await }
        });
        let close = Callback::new(move |_: ()| {
            delete_jam.dispatch(());
        });
        set_close(close);

        Effect::new(move |_| {
            if let Some(update) = message.get().or_else(move || {
                match initial_update.get().map(|r| r.deref().clone()) {
                    Some(Ok(update)) => Some(update),
                    Some(Err(e)) => {
                        warn!("Error getting initial update: {:#?}", e);
                        None
                    }
                    None => None,
                }
            }) {
                if let Some(users) = update.users {
                    set_users(Some(users));
                }
                if let Some(songs) = update.songs {
                    set_songs(Some(songs));
                }
                if let Some(votes) = update.votes {
                    set_votes(votes);
                }
                if !update.errors.is_empty() {
                    set_error_message(format!("Errors: {:#?}", update.errors));
                }
                if update.ended.is_some() {
                    close_ws();
                    let navigator = use_navigate();
                    navigator("/", NavigateOptions::default());
                }
                if update.search.is_some() {
                    warn!("Unexpected search update");
                }
                if update.position.is_some() {
                    warn!("Unexpected position update");
                }
                if update.current_song.is_some() {
                    warn!("Unexpected current song update");
                }
            }
        });
    });

    let close = Callback::new(move |_| {
        close.get_untracked().run(());
    });
    view! {
        <Modal visible=Signal::derive(move || {
            error_message.with(|e| !e.is_empty())
        })>
            {error_message}
            <button on:click=move |_| {
                set_error_message(String::new());
            }>"Close"</button>
        </Modal>
        <Title text=move || {
            jam
                .value()()
                .map(|jam| jam.map(|jam| jam.name.clone()))
                .unwrap_or(Ok(String::from("Host")))
                .unwrap_or_default()
        }/>
        <div class="host-page">
            <UsersBar close=close users kick_user/>
            <div class="center">
                <Player host_id set_song_position next_song/>
                <SongList
                    songs
                    votes
                    song_list_action=SongListAction::Remove(remove_song)
                    max_song_count=Signal::derive(move || {
                        jam
                            .value()()
                            .map(|jam| jam.map(|jam| jam.max_song_count))
                            .unwrap_or(Ok(0))
                            .unwrap_or_default()
                    })
                />

                <Share jam_id/>
            </div>
        </div>
    }
}

#[server]
async fn delete_jam(host_id: String) -> Result<(), ServerFnError> {
    use crate::model::{self, check_id_type, notify, AppState};
    let app_state = expect_context::<AppState>();
    let mut transaction = app_state.db.pool.begin().await?;
    let id = check_id_type(&host_id, &mut transaction).await?;
    if !id.is_host() {
        return Err(ServerFnError::Request("id is not a host id".to_string()));
    }
    model::delete_jam(&id.jam_id, &mut *transaction).await?;
    leptos_axum::redirect("/");
    use crate::model::real_time::Changed;
    notify(Changed::new().ended(), vec![], &id.jam_id, &mut transaction).await?;
    transaction.commit().await?;
    Ok(())
}

#[server]
pub async fn get_jam(jam_id: String) -> Result<Jam, ServerFnError> {
    use crate::model::functions::get_jam as get_jam_fn;
    use crate::model::AppState;
    let app_state = expect_context::<AppState>();
    let pool = &app_state.db.pool;
    match get_jam_fn(&jam_id, pool).await {
        Ok(jam) => Ok(jam),
        Err(e) => Err(ServerFnError::Request(e.to_string())),
    }
}

#[server]
pub async fn get_initial_update(id: String) -> Result<real_time::Update, ServerFnError> {
    use crate::model::{check_id_type, AppState};
    let app_state = expect_context::<AppState>();
    let mut transaction = app_state.db.pool.begin().await?;
    let id = check_id_type(&id, &mut transaction).await?;
    let update =
        real_time::Update::from_changed(real_time::Changed::all(), &id, &mut transaction).await;
    transaction.commit().await?;
    Ok(update)
}
