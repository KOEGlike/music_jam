use crate::general;
use crate::general::real_time::SearchResult;
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use wasm_bindgen::JsCast;

use crate::components::{error_template::*, song};
use crate::pages;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    view! {
        <Stylesheet id="leptos" href="/pkg/music_jam.css"/>

        <Title text="Welcome to Leptos"/>

        <Router fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! { <ErrorTemplate outside_errors/> }.into_view()
        }>
            <main>
                <Routes>
                    <Route path="/" view=pages::HomePage/>
                    <Route path="/create-host" view=pages::CreateHostPage/>
                    <Route path="/create-user/:id" view=pages::CreateUserPage/>
                    <Route path="/jam/host/:id" view=pages::HostPage/>
                    <Route path="/jam/:id" view=pages::UserPage/>
                    <Route path="/test-bar" view=UserBartTest/>
                    <Route path="/test-share" view=ShareTest/>
                    <Route path="/test-search" view=SearchTest/>
                    <Route path="/test-user-player" view=UserPlayerTest/>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
pub fn UserBartTest() -> impl IntoView {
    use crate::{components::UsersBar, general::User};
    use leptos::logging::*;

    let users = vec![
        User {
            id: "tb0k2ujdagg6bvvqeqlx2qgq".to_string(),
            jam_id: "niggaa".to_string(),
            name: "kaka".to_string(),
        },
        User {
            id: "coe7474an5pkiptmjls2bq0w".to_string(),
            jam_id: "niggaa".to_string(),
            name: "kakamaka".to_string(),
        },
        User {
            id: "bl0m5ktr6bs51hnbmkp8bs0c".to_string(),
            jam_id: "niggaa".to_string(),
            name: "kakamakanaka".to_string(),
        },
        User {
            id: "tb0k2ujdagg6bvvqeqlx2qgq".to_string(),
            jam_id: "niggaa".to_string(),
            name: "kaka".to_string(),
        },
        User {
            id: "coe7474an5pkiptmjls2bq0w".to_string(),
            jam_id: "niggaa".to_string(),
            name: "kakamaka".to_string(),
        },
        User {
            id: "bl0m5ktr6bs51hnbmkp8bs0c".to_string(),
            jam_id: "niggaa".to_string(),
            name: "kakamakanaka".to_string(),
        },
        User {
            id: "tb0k2ujdagg6bvvqeqlx2qgq".to_string(),
            jam_id: "niggaa".to_string(),
            name: "kaka".to_string(),
        },
        User {
            id: "coe7474an5pkiptmjls2bq0w".to_string(),
            jam_id: "niggaa".to_string(),
            name: "kakamaka".to_string(),
        },
        User {
            id: "bl0m5ktr6bs51hnbmkp8bs0c".to_string(),
            jam_id: "niggaa".to_string(),
            name: "kakamakanaka".to_string(),
        },
    ];
    let (users, set_users) = create_signal(Some(users));
    let kick_user = |id| {
        log!("kicking user {}", id);
    };
    let kick_user = Callback::from(kick_user);
    let close = Callback::new(move |_: ()| log!("close"));

    view! {
        <UsersBar close users kick_user/>
        <button on:click=move |_| { set_users(None) }>"loading"</button>
    }
}

#[component]
fn ShareTest() -> impl IntoView {
    use crate::components::Share;
    let jam_id = Signal::derive(|| "5Y8FXC".to_owned());
    view! { <Share jam_id/> }
}

#[component]
fn SearchTest() -> impl IntoView {
    use crate::components::user::Search;
    use leptos::logging::*;

    let song = general::Song {
        id: "lol".to_string(),
        user_id: None,
        name: "Yesterdayyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyy".to_string(),
        artists: vec![
            "Beatles".to_string(),
            "Beatles".to_string(),
            "Beatles".to_string(),
            "Beatles".to_string(),
            "Beatles".to_string(),
            "Beatles".to_string(),
        ],
        album: "Help!".to_string(),
        duration: 240,
        image_url: "https://i.scdn.co/image/ab67616d0000b273e3e3b64cea45265469d4cafa".to_string(),
        votes: general::Vote {
            votes: 0,
            have_you_voted: None,
        },
    };

    let songs = {
        let mut songs = Vec::new();
        for i in 0..10 {
            let mut song = song.clone();
            song.id = "a".repeat(i);
            songs.push(song);
        }
        songs
    };
    let (search_result, set_search_result) = create_signal(Some(SearchResult {
        search_id: "lol".to_string(),
        songs:songs.clone(),
    }));
    let search = move |id: (String, String)| {
        set_search_result(Some(SearchResult {
            songs:songs.clone(),
            search_id: id.1,
        }));
        log!("search with id:{}", id.0)
    };
    let search = Callback::from(search);

    let add_song = move |id| log!("add with id:{}", id);
    let add_song = Callback::from(add_song);
    let loaded = Signal::derive(|| true);

    view! { <Search search_result search add_song loaded/> }
}

#[component]
fn UserPlayerTest() -> impl IntoView {
    use crate::components::user::Player;
    use leptos::logging::*;

    let current_song = Signal::derive(move || {
        Some(general::Song {
            id: "lol".to_string(),
            user_id: None,
            name: "Yesterdayyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyy Yesterdayyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyy".to_string(),
            artists: vec!["Beatles".to_string()],
            album: "Help!".to_string(),
            duration: 240,
            image_url: "https://i.scdn.co/image/ab67616d0000b273e3e3b64cea45265469d4cafa"
                .to_string(),
            votes: general::Vote {
                votes: 0,
                have_you_voted: None,
            },
        })
    });
    let position = Signal::derive(|| 0.7);

    view! { <Player position current_song/> }
}

#[component]
fn Player() -> impl IntoView {
    use leptos::logging::log;
    use rust_spotify_web_playback_sdk::prelude as sp;

    let (current_song_name, set_current_song_name) = create_signal(String::new());

    let token = "BQAZ4oo4rm2B6DVW7SSrBgN9k9K6jw3Nk0UHKXnz9W1HU-oPWWPdXd3j7KjKFtO0dr399QrjEG-HkBd_dpEJi5VtijInn_WPPaffGK3TNHSnBxIiaeudshoGz3gDJsVpZNbqocpVc-7CCCe1kb0jGnDCLE2HpvWJYnkNR2w2pkkuxTedEg0KhfW40Ejs8tKyE7AsJS1oPeGIkmheR2p9SaIxn08C3ztxMXoi";

    let connect = create_action(|_| async {
        match sp::connect().await {
            Ok(_) => log!("connected"),
            Err(e) => log!("error {:?}", e),
        };
    });

    create_effect(move |_| {
        sp::init(
            || {
                log!("oauth was called");
                token.to_string()
            },
            move || {
                log!("ready");
                connect.dispatch(());

                sp::add_listener!("player_state_changed", move |state: sp::StateChange| {
                    log!("state changed: {:#?}", state);
                    set_current_song_name(state.track_window.current_track.name);
                })
                .unwrap();
            },
            "example player",
            1.0,
            false,
        );
    });

    let get_state = create_action(|_| async {
        let state = sp::get_current_state().await.unwrap();
        log!("state: {:#?}", state);
    });

    let activate_player = create_action(|_| async { sp::activate_element().await });

    view! {
        <h1>"Welcome to Leptos"</h1>
        <button on:click=move |_| activate_player.dispatch(())>"activate player"</button>

        {move || match activate_player.value().get() {
            Some(Ok(_)) => {
                view! {
                    <button on:click=move |_| get_state.dispatch(())>"log state in console"</button>
                    <p>"Current song: " {current_song_name}</p>
                }
                    .into_view()
            }
            Some(Err(e)) => view! { <p>"Error activating player: " {e}</p> }.into_view(),
            None => view! { <p>"Activating player..."</p> }.into_view(),
        }}
    }
}
