use gloo::timers::callback::Interval;
use leptos::{
    logging::{error, log},
    prelude::*,
    *,
};
use rust_spotify_web_playback_sdk::prelude as sp;

use crate::{app::pages::host, components::create};

#[component]
pub fn Player(
    #[prop(into)] host_id: Signal<Option<String>>,
    #[prop(into)] top_song_id: Signal<Option<String>>,
    #[prop(into)] reset_votes: Callback<()>,
) -> impl IntoView {
    let (player_is_connected, set_player_is_connected) = create_signal(false);

    let top_song_id = create_memo(move |_| top_song_id());
    //let (current_song_id, set_current_song_id) = create_signal(String::new());
    //let current_song_id = create_memo(move |_| current_song_id());

    let (image_url, set_image_url) = create_signal(String::new());
    let (song_name, set_song_name) = create_signal(String::new());
    let (artists, set_artists) = create_signal(String::new());
    let (song_length, set_song_length) = create_signal(0);
    let (song_position, set_song_position) = create_signal(0);
    let (playing, set_playing) = create_signal(false);

    let on_update = move |state_change: sp::StateChange| {
        set_song_position(state_change.position);
        set_playing(!state_change.paused);
        set_song_length(state_change.track_window.current_track.duration_ms);
        set_image_url(
            state_change.track_window.current_track.album.images[0]
                .url
                .clone(),
        );
        set_song_name(state_change.track_window.current_track.name);

        set_artists(
            state_change
                .track_window
                .current_track
                .artists
                .into_iter()
                .map(|a| a.name)
                .collect::<Vec<_>>()
                .join(", "),
        );
    };

    let token = {
        create_action(move |_: &()| {
            log!("getting token for host_id:{:?}", host_id.get_untracked());
            async move {
                if let Some(host_id) = host_id.get_untracked() {
                    get_access_token(host_id).await
                } else {
                    error!("host id is empty, act");
                    Err(ServerFnError::Request("Host id is empty".to_string()))
                }
            }
        })
    };

    let switch_device = {
        create_action(move |device_id: &String| {
            let device_id = device_id.clone();
            async move {
                if let Some(host_id) = host_id.get_untracked() {
                    if let Err(e) = change_playback_device(device_id, host_id).await {
                        error!("Error switching device: {:?}", e);
                    }
                } else {
                    error!("host id is empty, act");
                }
            }
        })
    };

    let connect = create_action(move |_: &()| async move { sp::connect().await });

    create_effect(move |_| match connect.value().get() {
        Some(Ok(_)) => {
            set_player_is_connected(true);
        }
        Some(Err(e)) => {
            error!("error while connecting to spotify:{:?}", e);
        }
        None => {}
    });

    create_effect(move |_| {
        if !sp::player_ready() && host_id.with(Option::is_some) {
            if let Some(Ok(token_value)) = token.value().get() {
                log!("initializing player with token: {:?}", token_value);
                sp::init(
                    move || {
                        token.dispatch(());
                        token_value.access_token.clone()
                    },
                    move || {
                        
                        sp::add_listener!("player_state_changed", on_update).unwrap();
                        sp::add_listener!("ready", move |player: sp::Player| {
                            switch_device.dispatch(player.device_id);
                        })
                        .unwrap();
                        connect.dispatch(());
                    },
                    "jam",
                    1.0,
                    false,
                );
            } else {
                token.dispatch(());
            }
        }
    });

    let is_loaded = create_memo(move |_| {
        player_is_connected() && top_song_id.with(Option::is_some) && host_id.with(Option::is_some)
    });

    create_effect(move |_| {
        log!("player is connected:{}", is_loaded());
    });

    view! {
        <Show when=is_loaded fallback=|| "loading.......">
            {move || {
                let host_id = host_id().unwrap();
                log!("is ready, host_id:{}", host_id);
                let play_song = {
                    let host_id = host_id.clone();
                    create_action(move |song_id: &String| {
                        let host_id = host_id.clone();
                        let song_id = song_id.clone();
                        async move {
                            if let Err(e) = play_song(song_id, host_id).await {
                                error!("Error playing song: {:?}", e);
                            }
                        }
                    })
                };
                let toggle_play = create_action(move |_: &()| async {
                    if let Err(e) = sp::toggle_play().await {
                        error!("Error toggling play: {:?}", e);
                    }
                });
                let position_update = create_action(move |_: &()| async move {
                    if is_loaded.get_untracked() {
                        if let Ok(Some(state)) = sp::get_current_state().await {
                            set_song_position(state.position);
                        }
                    }
                });
                if cfg!(any(target_arch = "wasm32", target_os = "unknown")) {
                    let position_update = Interval::new(
                        100,
                        move || {
                            position_update.dispatch(());
                        },
                    );
                    position_update.forget();
                }
                let can_go_to_next_song = create_memo(move |_| {
                    (song_position() as f32 / song_length() as f32) > 0.995
                });
                create_effect(move |_| {
                    if can_go_to_next_song() {
                        if let Some(song_id) = top_song_id.get() {
                            play_song.dispatch(song_id);
                        } else {
                            toggle_play.dispatch(());
                        }
                    }
                });
                
                view! {
                    <div class="player">
                        <img prop:src=image_url alt="the album cover of the current song"/>

                        <div class="info">
                            <div class="title">{song_name}</div>
                            <div class="artist">{artists}</div>
                        </div>

                        <div class="progress">
                            <div class="bar">
                                <div
                                    class="position"
                                    style:width=move || {
                                        let percentage = if song_length() == 0 {
                                            0.0
                                        } else {
                                            (song_position() as f64 / song_length() as f64) * 100.0
                                        };
                                        format!("{}%", percentage)
                                    }
                                >
                                </div>
                            </div>
                            <div class="times">
                                <div>{move || { millis_to_min_sec(song_position() as u32) }}</div>
                                <div>{move || millis_to_min_sec(song_length() as u32)}</div>
                            </div>
                        </div>

                        <button
                            on:click=move |_| toggle_play.dispatch(())
                            class="play-pause"
                            title="play-pause"
                        >
                            {move || match playing() {
                                true => {
                                    view! {
                                        <svg
                                            viewBox=icondata::FaPauseSolid.view_box
                                            inner_html=icondata::FaPauseSolid.data
                                            class="pause"
                                        ></svg>
                                    }
                                }
                                false => {
                                    view! {
                                        <svg
                                            viewBox=icondata::BsPlayFill.view_box
                                            inner_html=icondata::BsPlayFill.data
                                            class="play"
                                        ></svg>
                                    }
                                }
                            }}

                        </button>
                    </div>
                }
            }}

        </Show>
    }
}

fn millis_to_min_sec(millis: u32) -> String {
    let seconds = millis / 1000;
    let minutes = seconds / 60;
    let seconds = seconds % 60;
    format!("{:01}:{:02}", minutes, seconds)
}

#[server]
async fn play_song(song_id: String, host_id: String) -> Result<(), ServerFnError> {
    use crate::general::*;
    let app_state = expect_context::<AppState>();
    let pool = &app_state.db.pool;
    let credentials = app_state.spotify_credentials;

    let jam_id = match check_id_type(&host_id, pool).await {
        Ok(IdType::Host(id)) => id.jam_id,
        Ok(IdType::User(_)) => {
            leptos_axum::redirect("/");
            return Err(ServerFnError::Request(
                "the id was found, but it belongs to a user".to_string(),
            ));
        }
        Err(e) => return Err(ServerFnError::ServerError(e.to_string())),
    };

    if let Err(e) = crate::general::play_song(&song_id, &jam_id, pool, credentials).await {
        return Err(ServerFnError::ServerError(e.into()));
    };

    Ok(())
}

#[server]
async fn change_playback_device(device_id: String, host_id: String) -> Result<(), ServerFnError> {
    use crate::general::*;
    let app_state = expect_context::<AppState>();
    let pool = &app_state.db.pool;
    let credentials = app_state.spotify_credentials;

    let jam_id = match check_id_type(&host_id, pool).await {
        Ok(IdType::Host(id)) => id.jam_id,
        Ok(IdType::User(_)) => {
            leptos_axum::redirect("/");
            return Err(ServerFnError::Request(
                "the id was found, but it belongs to a user".to_string(),
            ));
        }
        Err(e) => return Err(ServerFnError::ServerError(e.to_string())),
    };

    if let Err(e) = switch_playback_to_device(&device_id, &jam_id, pool, credentials).await {
        return Err(ServerFnError::ServerError(e.into()));
    };

    Ok(())
}

#[server]
async fn get_access_token(host_id: String) -> Result<rspotify::Token, ServerFnError> {
    use crate::general::*;
    let app_state = expect_context::<AppState>();
    let pool = &app_state.db.pool;
    let credentials = app_state.spotify_credentials;

    let jam_id = check_id_type(&host_id, pool).await;
    let jam_id = match jam_id {
        Ok(id) => id,
        Err(sqlx::Error::RowNotFound) => {
            leptos_axum::redirect("/");
            return Err(ServerFnError::Request("Host not found".to_string()));
        }
        Err(e) => return Err(ServerFnError::ServerError(e.to_string())),
    };
    let jam_id = match jam_id {
        IdType::Host(id) => id.jam_id,
        IdType::User(_) => {
            leptos_axum::redirect("/");
            return Err(ServerFnError::Request(
                "the id was found, but it belongs to a user".to_string(),
            ));
        }
    };

    let token = match crate::general::get_access_token(pool, &jam_id, credentials).await {
        Ok(token) => token,
        Err(e) => return Err(ServerFnError::ServerError(e.into())),
    };

    Ok(token)
}
