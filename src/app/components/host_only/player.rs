use leptos::{
    logging::{error, log},
    prelude::*,
    *,
};
use rust_spotify_web_playback_sdk::prelude as sp;

use crate::{app::general::types::Song, components::create};

#[component]
pub fn Player<F>(
    host_id: String,
    #[prop(into)] top_song: Signal<Option<Song>>,
    reset_votes: F,
) -> impl IntoView
where
    F: Fn() + 'static,
{
    let (player_is_connected, set_player_is_connected) = create_signal(false);
    let token = create_action(move |_: &()| {
        let host_id = host_id.clone();
        async move { get_access_token(host_id).await }
    });

    token.dispatch(());

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

    let toggle_play = create_action(move |_: &()| async {
        if let Err(e) = sp::toggle_play().await {
            error!("Error toggling play: {:?}", e);
        }
    });

    let is_loaded = move || {
        let x = player_is_connected() || top_song.with(|song| song.is_some());
        if x {
            log!("player is connected");
        } else {
            log!("player is not connected");
        }
        x
    };
    let song_url = move || {
        top_song.with(|song| match song.as_ref() {
            Some(song) => song.image.url.clone(),
            None => String::new(),
        })
    };
    let song_name = move || {
        top_song.with(|song| match song.as_ref() {
            Some(song) => song.name.clone(),
            None => String::new(),
        })
    };
    let artists = move || {
        top_song.with(|song| match song.as_ref() {
            Some(song) => song.artists.join(", "),
            None => String::new(),
        })
    };
    let song_length = move || {
        top_song.with(|song| match song.as_ref() {
            Some(song) => song.duration,
            None => 0,
        })
    };
    let (song_position, set_song_position) = create_signal(0);
    let (playing, set_playing) = create_signal(false);

    create_effect(move |_| {
        if !sp::player_ready() {
            if let Some(Ok(token_value)) = token.value().get() {
                sp::init(
                    move || {
                        token.dispatch(());
                        token_value.access_token.clone()
                    },
                    move || {
                        sp::add_listener!(
                            "player_state_changed",
                            move |state_change: sp::StateChange| {
                                set_song_position(state_change.position);
                                set_playing(!state_change.paused);
                                log!("state change: {:#?}", state_change);
                            }
                        );
                        connect.dispatch(());
                    },
                    "jam",
                    1.0,
                    false,
                );
            }
        }
    });

    view! {
        <Show when=is_loaded fallback=|| "loading.......">
            <div class="player">
                <img prop:src=song_url/>

                <div class="info">
                    <div class="title">{song_name}</div>
                    <div class="artist">{artists}</div>
                </div>

                <div class="progress">
                    <div class="bar">
                        <div class="position"></div>
                    </div>
                    <div class="times">
                        <div>{song_position}</div>
                        <div>{song_length}</div>
                    </div>
                </div>

                <button on:click=move |_| toggle_play.dispatch(()) class="play-pause">
                    {move || match playing() {
                        true => {
                            view! {
                                <svg
                                    width="60"
                                    height="54"
                                    viewBox=icondata::FaPauseSolid.view_box
                                    inner_html=icondata::FaPauseSolid.data
                                    class="pause"
                                ></svg>
                            }
                        }
                        false => {
                            view! {
                                <svg
                                    width="71"
                                    height="71"
                                    viewBox=icondata::BsPlayFill.view_box
                                    inner_html=icondata::BsPlayFill.data
                                    class="play"
                                ></svg>
                            }
                        }
                    }}

                </button>
            </div>
        </Show>
    }
}

#[server]
async fn get_access_token(host_id: String) -> Result<rspotify::Token, ServerFnError> {
    use crate::app::general::*;
    let app_state = expect_context::<AppState>();
    let pool = &app_state.db.pool;
    let reqwest_client = &app_state.reqwest_client;

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

    let token = match get_access_token(pool, &jam_id, reqwest_client).await {
        Ok(token) => token,
        Err(e) => return Err(ServerFnError::ServerError(e.into())),
    };

    Ok(token)
}
