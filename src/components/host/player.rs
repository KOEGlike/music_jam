use std::time::Duration;

use crate::components::general::{self, modal::*};
use leptos::{
    either::*,
    logging::{error, log},
    prelude::*,
    *,
};
use leptos_router::hooks::use_navigate;
use rust_spotify_web_playback_sdk::prelude as sp;
use task::spawn_local;

use crate::model;

#[component]
pub fn Player(
    #[prop(into)] host_id: Signal<Option<String>>,
    #[prop(into)] set_song_position: Callback<f32>,
) -> impl IntoView {
    let (error_message, set_error_message) = signal(String::new());

    let set_global_song_position = set_song_position;

    let (player_is_connected, set_player_is_connected) = signal(false);

    let (current_song, set_current_song) = signal(None::<model::Song>);
    let (playing, set_playing) = signal(false);

    let on_update = move |state_change: sp::StateChange| {
        set_playing(!state_change.paused);
        let mut current_song = state_change.track_window.current_track;
        set_current_song(Some(model::Song {
            id: None,
            spotify_id: current_song.id,
            user_id: None,
            name: current_song.name,
            artists: current_song.artists.into_iter().map(|a| a.name).collect(),
            album: current_song.album.name,
            duration: current_song.duration_ms,
            image_url: current_song.album.images.remove(0).url,
            votes: model::Vote {
                votes: 0,
                have_you_voted: None,
            },
        }));
    };

    let switch_device = move |device_id: String| {
        spawn_local(async move {
            if let Some(host_id) = host_id.try_get_untracked() {
                if let Some(host_id) = host_id {
                    if let Err(e) = change_playback_device(device_id, host_id).await {
                        set_error_message(format!("Error switching device: {:?}", e));
                    }
                } else {
                    use leptos_router::NavigateOptions;
                    set_error_message("host id is empty, act".into());
                    use_navigate()("/", NavigateOptions::default());
                }
            }
        })
    };

    let connect = move || {
        spawn_local(async move {
            match sp::connect().await {
                Ok(_) => {
                    set_player_is_connected(true);
                }
                Err(e) => {
                    set_error_message(format!("error while connecting to spotify:{:?}", e));
                }
            }
        })
    };

    let get_token = {
        let host_id = host_id.clone();
        let set_error_message = set_error_message.clone();
        async move || {
            use gloo::timers::future::sleep;

            let mut token = Err(ServerFnError::Request("token error".to_string()));
            let mut res = None;
            while token.is_err() {
                token = {
                    if let Some(host_id) = host_id.get_untracked() {
                        get_access_token(host_id).await
                    } else {
                        use leptos_router::NavigateOptions;
                        use_navigate()("/", NavigateOptions::default());
                        error!("host id is empty");
                        Err(ServerFnError::Request("host id is empty".to_string()))
                    }
                };
                match &token {
                    Err(e) => {
                        set_error_message(format!("Error getting token: {:?}", e));
                        error!("Error getting token: {:?}", e);
                        sleep(Duration::from_secs(2)).await;
                    }
                    Ok(token) => {
                        res = Some(token.clone());
                        break;
                    }
                }
            }
            if let Some(token) = res {
                token
            } else {
                panic!("token error");
            }
        }
    };

    let get_token_action = Action::new_local(move |_: &()| async move { get_token().await });

    Effect::new(move || {
        spawn_local(async move {
            if sp::player_ready() && !host_id.with(Option::is_some) {
                return;
            }
            let token = get_token().await;
            get_token_action.dispatch(());

            log!("initializing player with token: {:?}", token);
            sp::init(
                move || {
                    let t = match get_token_action.value().get_untracked() {
                        Some(t) => t,
                        None => token.clone(),
                    };
                    t.access_token
                },
                move || {
                    if let Err(e) = sp::add_listener!("player_state_changed", on_update) {
                        set_error_message(format!("Error adding listener: {:?}", e));
                    }
                    if let Err(e) = sp::add_listener!("ready", move |player: sp::Player| {
                        switch_device(player.device_id);
                    }) {
                        set_error_message(format!("Error adding listener: {:?}", e));
                    }
                    log!("player ready");
                    connect();
                },
                "jam",
                1.0,
                false,
            );
        });
    });

    let is_loaded = Memo::new(move |_| player_is_connected() && host_id.with(Option::is_some));

    Effect::new(move |_| {
        log!("player is connected:{}", is_loaded());
    });

    let toggle_play = move || {
        spawn_local(async move {
            if let Err(e) = sp::toggle_play().await {
                set_error_message(format!("Error toggling play: {:?}", e));
            }
        })
    };

    let calculate_percentage = move |current_time: i32| {
        current_time as f32
            / current_song.with_untracked(|s| s.as_ref().map(|s| s.duration).unwrap_or(1)) as f32
    };

    let (position_percentage, set_position_percentage) = signal(0.0);
    let position_update = move || {
        spawn_local(async move {
            if is_loaded.get_untracked() {
                if let Ok(Some(state)) = sp::get_current_state().await {
                    let percentage = calculate_percentage(state.position);
                    if !(position_percentage() > 0.999 && percentage > 0.999) {
                        set_global_song_position.run(percentage);
                        set_position_percentage(percentage);
                    }
                }
            }
        })
    };
    Effect::new(move |_| {
        if is_loaded() {
            leptos_use::use_interval_fn(
                move || {
                    position_update();
                },
                10,
            );
        }
    });

    on_cleanup(move || {
        if cfg!(target_arch = "wasm32") {
            if let Err(e) = sp::disconnect() {
                error!("Error disconnecting player: {:?}", e);
            };
        }
    });

    view! {
        <Modal visible=Signal::derive(move || {
            error_message.with(|e| !e.is_empty())
        })>
            {error_message}
            <button on:click=move |_| {
                set_error_message("".into());
            }>"close"</button>
        </Modal>
        <general::Player current_song position=position_percentage>
            <button
                on:click=move |_| {
                    toggle_play();
                }

                class="play-pause"
                title=move || match playing() {
                    true => "pause",
                    false => "play",
                }
            >

                {move || match playing() {
                    true => {
                        Either::Left(
                            view! {
                                <svg
                                    viewBox=icondata::FaPauseSolid.view_box
                                    inner_html=icondata::FaPauseSolid.data
                                    class="pause"
                                ></svg>
                            },
                        )
                    }
                    false => {
                        Either::Right(
                            view! {
                                <svg
                                    viewBox=icondata::BsPlayFill.view_box
                                    inner_html=icondata::BsPlayFill.data
                                    class="play"
                                ></svg>
                            },
                        )
                    }
                }}

            </button>
        </general::Player>
    }
}

#[server]
async fn change_playback_device(
    device_id: String,
    host_id: String,
) -> Result<(), ServerFnError<String>> {
    use crate::model::*;
    let app_state = expect_context::<AppState>();
    let mut transaction =
        app_state.db.pool.begin().await.map_err(|e| {
            ServerFnError::ServerError(format!("error starting transaction: {}", e))
        })?;
    let credentials = app_state.spotify_credentials;

    let jam_id = match model::check_id_type(&host_id, &mut transaction).await {
        Ok(id) => match id.id {
            IdType::Host(_) => id.jam_id,
            _ => {
                leptos_axum::redirect("/");
                return Err(ServerFnError::Request(
                    "the id was found, but it belongs to a user".to_string(),
                ));
            }
        },
        Err(e) => return Err(ServerFnError::ServerError(e.to_string())),
    };

    if let Err(e) =
        model::switch_playback_to_device(&device_id, &jam_id, &mut transaction, credentials).await
    {
        return Err(ServerFnError::ServerError(e.to_string()));
    };

    transaction
        .commit()
        .await
        .map_err(|e| ServerFnError::ServerError(format!("error committing transaction: {}", e)))?;

    Ok(())
}

#[server]
async fn get_access_token(host_id: String) -> Result<rspotify::Token, ServerFnError<String>> {
    use crate::model::*;

    let app_state = expect_context::<AppState>();
    let mut transaction =
        app_state.db.pool.begin().await.map_err(|e| {
            ServerFnError::ServerError(format!("error starting transaction: {}", e))
        })?;
    let credentials = app_state.spotify_credentials;

    let id = check_id_type(&host_id, &mut transaction).await;
    let id = match id {
        Ok(id) => id,
        Err(e) => return Err(ServerFnError::ServerError(e.to_string())),
    };
    let jam_id = if id.is_host() {
        id.jam_id
    } else {
        leptos_axum::redirect("/");
        return Err(ServerFnError::Request(
            "the id was found, but it belongs to a user".to_string(),
        ));
    };

    let token = match crate::model::get_access_token(&mut transaction, &jam_id, credentials).await {
        Ok(token) => token,
        Err(e) => return Err(ServerFnError::ServerError(e.into())),
    };

    transaction
        .commit()
        .await
        .map_err(|e| ServerFnError::ServerError(format!("error committing transaction: {}", e)))?;

    Ok(token)
}
