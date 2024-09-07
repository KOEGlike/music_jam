use crate::components::user;
use gloo::timers::callback::Interval;
use leptos::{
    logging::{error, log},
    prelude::*,
    *,
};
use rust_spotify_web_playback_sdk::prelude as sp;

use crate::model;

#[component]
pub fn Player(
    #[prop(into)] host_id: Signal<Option<String>>,
    #[prop(into)] set_song_position: Callback<f32>,
    #[prop(into)] next_song: Callback<()>,
) -> impl IntoView {
    let set_global_song_position = set_song_position;

    let (player_is_connected, set_player_is_connected) = create_signal(false);

    //let (current_song_id, set_current_song_id) = create_signal(String::new());
    //let current_song_id = create_memo(move |_| current_song_id());

    let (current_song, set_current_song) = create_signal(None::<model::Song>);
    let (song_position, set_song_position) = create_signal(0);
    let (playing, set_playing) = create_signal(false);

    let on_update = move |state_change: sp::StateChange| {
        set_song_position(state_change.position);
        set_playing(!state_change.paused);
        let mut current_song = state_change.track_window.current_track;
        set_current_song(Some(model::Song {
            id: current_song.id,
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

    let is_loaded = create_memo(move |_| player_is_connected() && host_id.with(Option::is_some));

    create_effect(move |_| {
        log!("player is connected:{}", is_loaded());
    });

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

    create_effect(move |_| {
        if is_loaded() {
            let position_update = Interval::new(100, move || {
                position_update.dispatch(());
            });
            position_update.forget();
        }
    });

    let position_percentage = Signal::derive(move || {
        song_position() as f32
            / current_song.with(|s| s.as_ref().map(|s| s.duration).unwrap_or(1)) as f32
    });

    create_effect(move |_| {
        set_global_song_position(position_percentage());
    });

    let can_go_to_next_song = create_memo(move |_| position_percentage() > 0.995);

    create_effect(move |_| {
        if can_go_to_next_song() && player_is_connected() {
            next_song(());
        }
    });

    on_cleanup(move || {
        if let Err(e) = sp::disconnect() {
            error!("Error disconnecting player: {:?}", e);
        };
    });

    view! {
        <user::Player current_song position=position_percentage>
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
        </user::Player>
    }
}

#[server]
async fn change_playback_device(device_id: String, host_id: String) -> Result<(), ServerFnError> {
    use crate::model::*;
    let app_state = expect_context::<AppState>();
    let pool = &app_state.db.pool;
    let credentials = app_state.spotify_credentials;

    let jam_id = match check_id_type(&host_id, pool).await {
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

    if let Err(e) = switch_playback_to_device(&device_id, &jam_id, pool, credentials).await {
        return Err(ServerFnError::ServerError(e.into()));
    };

    Ok(())
}

#[server]
async fn get_access_token(host_id: String) -> Result<rspotify::Token, ServerFnError> {
    use crate::model::*;

    let app_state = expect_context::<AppState>();
    let pool = &app_state.db.pool;
    let credentials = app_state.spotify_credentials;

    let id = check_id_type(&host_id, pool).await;
    let id = match id {
        Ok(id) => id,
        Err(sqlx::Error::RowNotFound) => {
            leptos_axum::redirect("/");
            return Err(ServerFnError::Request("Host not found".to_string()));
        }
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

    let token = match crate::model::get_access_token(pool, &jam_id, credentials).await {
        Ok(token) => token,
        Err(e) => return Err(ServerFnError::ServerError(e.into())),
    };

    Ok(token)
}
