use crate::components::{host::Player, Share, SongList, SongListAction, UsersBar};
use crate::model::types::*;
use codee::binary::MsgpackSerdeCodec;
use gloo::storage::{LocalStorage, Storage};
use leptos::{logging::*, prelude::*, *};
use leptos_meta::Title;
use leptos_router::{use_navigate, use_params_map, NavigateOptions};
use leptos_use::{use_websocket, UseWebSocketReturn};
use real_time::Changed;

#[component]
pub fn HostPage() -> impl IntoView {
    let (host_id, set_host_id) = create_signal(String::new());
    let host_id = create_memo(move |_| {
        if host_id.with(String::is_empty) {
            None
        } else {
            Some(host_id.get_untracked())
        }
    });
    create_effect(move |_| {
        let host_id: String = LocalStorage::get("host_id").unwrap_or_default();
        if host_id.is_empty() {
            let navigator = use_navigate();
            navigator("/", NavigateOptions::default());
        }
        set_host_id(host_id);
    });

    let jam_id = move || use_params_map().with(|params| params.get("id").cloned());
    let jam_id = Signal::derive(jam_id);
    let (jam_id_new, set_jam_id) = create_signal(String::new());
    create_effect(move |_| {
        let jam_id = jam_id();
        if let Some(jam_id) = jam_id {
            set_jam_id(jam_id);
        }
    });
    create_effect(move |_| log!("jam_id:{:?}", jam_id()));
    let jam_id = jam_id_new;

    let jam = create_action(move |_: &()| async move {
        let jam_id = jam_id.get_untracked();

        get_jam(jam_id).await
    });
    create_effect(move |_| jam.dispatch(()));
    create_effect(move |_| {
        if let Some(jam_val) = jam.value().get() {
            if jam_val.is_err() {
                jam.dispatch(());
            }
        }
    });

   

    let (users, set_users) = create_signal(None);
    let (songs, set_songs) = create_signal(None::<Vec<Song>>);
    let (votes, set_votes) = create_signal(Votes::new());

    let (send_request, set_send_request) = create_signal(Callback::new(|_: real_time::Request| {
        warn!("wanted to send a message to ws, but the ws is not ready yet");
    }));
    let (close, set_close) = create_signal(Callback::new(|_: ()| {
        warn!("wanted to close ws, but the ws is not ready yet");
    }));


    let remove_song = move |id| {
        let request = real_time::Request::RemoveSong { song_id: id };
        send_request.get_untracked()(request);
    };
    let remove_song = Callback::new(remove_song);

    let request_update = move || {
        let request = real_time::Request::Update;
        send_request.get_untracked()(request);
    };

    let kick_user = move |id| {
        let request = real_time::Request::KickUser { user_id: id };
        send_request.get_untracked()(request);
    };
    let kick_user = Callback::new(kick_user);

    let next_song = move |_:()| {
        let request = real_time::Request::NextSong;
        send_request.get_untracked()(request);
    };
    let next_song = Callback::new(next_song);

    let set_song_position = move |percentage| {
        let request = real_time::Request::Position { percentage };
        send_request.get_untracked()(request);
    };
    let set_song_position = Callback::new(set_song_position);

    create_effect(move |_| log!("host_id:{:?}", host_id()));

    create_effect(move |_| {
        let host_id = match host_id() {
            Some(host_id) => host_id,
            None => return,
        };

        let UseWebSocketReturn {
            //ready_state,
            message,
            close: close_ws,
            send,
            ..
        } = use_websocket::<real_time::Request,real_time::Update, MsgpackSerdeCodec>(&format!(
            "/socket?id={}",
            host_id
        ));

        

        let send_request = Callback::new(move|request| send(&request));
        set_send_request(send_request);

        let delete_jam = create_action(move |_: &()| {
            let host_id = host_id.clone();
            async move { delete_jam(host_id).await }
        });
        let close = Callback::new(move |_: ()| {
            delete_jam.dispatch(());
        });
        set_close(close);

        create_effect(move |_| {
            if let Some(update) = message() {
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
                    error!("Errors: {:#?}", update.errors);
                }
                if update.ended.is_some() {
                    close_ws();
                    let navigator = use_navigate();
                    navigator("/", NavigateOptions::default());
                }
                if update.search.is_some() {
                    error!("Unexpected search update");
                }
                if update.position.is_some() {
                    error!("Unexpected position update");
                }
                if update.current_song.is_some() {
                    error!("Unexpected current song update");
                }
            }
        });
    });

    let close = Callback::new(move |_| {
        close.get_untracked()(());
    });
    view! {
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
                    request_update
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
    let pool = &app_state.db.pool;
    let id = check_id_type(&host_id, pool).await?;
    if !id.is_host() {
        return Err(ServerFnError::Request("id is not a host id".to_string()));
    }
    model::delete_jam(&id.jam_id, pool).await?;
    notify(Changed::new().ended(), vec![], &id.jam_id, pool).await?;
    leptos_axum::redirect("/");
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
