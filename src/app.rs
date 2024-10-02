use crate::model;
use crate::model::real_time::SearchResult;
use leptos::{
    prelude::*,
    spawn::{self, spawn_local},
};
use leptos_meta::*;
use leptos_router::{
    components::{Route, Router, Routes},
    *,
};

use crate::pages;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <link rel="stylesheet" href="/pkg/music_jam.css"/>
                <title>"Music Jam"</title>
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    let (is_loading, set_is_loading) = signal(false);
    Effect::new(move |_| {
        set_is_loading(false);
    });
    view! {
        <Router>
            <main style:display=move || if is_loading() { "none" } else { "inline-block" }>
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=path!("/") view=pages::HomePage/>
                    <Route path=path!("/create-host") view=pages::CreateHostPage/>
                    <Route path=path!("/create-user/:id") view=pages::CreateUserPage/>
                    <Route path=path!("/jam/host/:id") view=pages::HostPage/>
                    <Route path=path!("/jam/:id") view=pages::UserPage/>
                    <Route path=path!("/test-bar") view=UserBartTest/>
                    <Route path=path!("/test-share") view=ShareTest/>
                    <Route path=path!("/test-search") view=SearchTest/>
                    <Route path=path!("/test-user-player") view=UserPlayerTest/>
                    <Route path=path!("/test-player") view=Player/>
                    <Route path=path!("/test-song-list") view=SongListTest/>
                </Routes>
            </main>
            <div
                class="loading-indicator"
                style:display=move || if is_loading() { "inline-block" } else { "none" }
            >
                <p>"Loading..."</p>
            </div>
        </Router>
    }
}

#[component]
pub fn SongListTest() -> impl IntoView {
    use crate::components::SongList;
    use crate::model::Song;
    use leptos::logging::*;

    let song = Song {
        id: Some("lol".to_string()),
        spotify_id: "lol".to_string(),
        user_id: None,
        name: "Yesterdayyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyy".to_string(),
        artists: vec!["Beatles".to_string()],
        album: "Help!".to_string(),
        duration: 240,
        image_url: "https://i.scdn.co/image/ab67616d0000b273e3e3b64cea45265469d4cafa".to_string(),
        votes: model::Vote {
            votes: 0,
            have_you_voted: None,
        },
    };
    let songs = {
        let mut songs = Vec::new();
        for i in 0..10 {
            let mut song = song.clone();
            song.id = Some("a".repeat(i));
            songs.push(song);
        }
        songs
    };
    use crate::components::SongListAction;
    let (songs, set_songs) = signal(Some(songs));
    let (votes, set_votes) = signal(model::Votes::new());
    let max_song_count = Signal::derive(|| 10);
    let song_list_action = SongListAction::Vote {
        add_vote: Callback::new(|id| log!("add vote with id:{}", id)),
        remove_vote: Callback::new(|id| log!("remove vote with id:{}", id)),
        remove_song: Callback::new(|id| log!("remove song with id:{}", id)),
    };
    view! { <SongList songs votes max_song_count song_list_action/> }
}

#[component]
pub fn UserBartTest() -> impl IntoView {
    use crate::{components::UsersBar, model::User};
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
    let (users, set_users) = signal(Some(users));
    let kick_user = |id| {
        log!("kicking user {}", id);
    };
    let kick_user = Callback::new(kick_user);
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

    let song = model::Song {
        id: Some("lol".to_string()),
        spotify_id: "lol".to_string(),
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
        votes: model::Vote {
            votes: 0,
            have_you_voted: None,
        },
    };

    let songs = {
        let mut songs = Vec::new();
        for i in 0..10 {
            let mut song = song.clone();
            song.id = Some("a".repeat(i));
            songs.push(song);
        }
        songs
    };
    let (search_result, set_search_result) = signal(Some(SearchResult {
        search_id: "lol".to_string(),
        songs: songs.clone(),
    }));
    let search = move |id: (String, String)| {
        set_search_result(Some(SearchResult {
            songs: songs.clone(),
            search_id: id.1,
        }));
        log!("search with id:{}", id.0)
    };
    let search = Callback::new(search);

    let add_song = move |id| log!("add with id:{}", id);
    let add_song = Callback::new(add_song);
    let loaded = Signal::derive(|| true);

    view! { <Search search_result search add_song loaded/> }
}

#[component]
fn UserPlayerTest() -> impl IntoView {
    use crate::components::general::Player;

    let (current_song, _) = signal(Some(model::Song {
        id: Some("lol".to_string()),
        spotify_id: "lol".to_string(),
        user_id: None,
        name:
            "Yesterdayyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyy Yesterdayyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyy"
                .to_string(),
        artists: vec!["Beatles".to_string()],
        album: "Help!".to_string(),
        duration: 240,
        image_url: "https://i.scdn.co/image/ab67616d0000b273e3e3b64cea45265469d4cafa".to_string(),
        votes: model::Vote {
            votes: 0,
            have_you_voted: None,
        },
    }));
    let position = Signal::derive(|| 0.7);

    view! { <Player position current_song=current_song/> }
}

#[component]
fn Player() -> impl IntoView {
    use leptos::either::EitherOf3;
    use leptos::logging::log;
    use rust_spotify_web_playback_sdk::prelude as sp;

    let (current_song_name, set_current_song_name) = signal(String::new());

    let token = "BQAZ4oo4rm2B6DVW7SSrBgN9k9K6jw3Nk0UHKXnz9W1HU-oPWWPdXd3j7KjKFtO0dr399QrjEG-HkBd_dpEJi5VtijInn_WPPaffGK3TNHSnBxIiaeudshoGz3gDJsVpZNbqocpVc-7CCCe1kb0jGnDCLE2HpvWJYnkNR2w2pkkuxTedEg0KhfW40Ejs8tKyE7AsJS1oPeGIkmheR2p9SaIxn08C3ztxMXoi";

    let connect = move || {
        spawn::spawn_local(async move {
            match sp::connect().await {
                Ok(_) => log!("connected"),
                Err(e) => log!("error {:?}", e),
            };
        })
    };

    Effect::new(move |_| {
        sp::init(
            || {
                log!("oauth was called");
                token.to_string()
            },
            move || {
                log!("ready");
                connect();

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

    let get_state = move || {
        spawn_local(async move {
            let state = sp::get_current_state().await.unwrap();
            log!("state: {:#?}", state);
        })
    };

    let (err, set_err) = signal(Option::None);
    let activate_player = move || {
        spawn_local(async move {
            set_err(Some(sp::activate_element().await));
        })
    };

    view! {
        <h1>"Welcome to Leptos"</h1>
        <button on:click=move |_| {
            activate_player();
        }>"activate player"</button>

        {move || match err() {
            Some(Ok(_)) => {
                EitherOf3::A(
                    view! {
                        <button on:click=move |_| {
                            get_state();
                        }>"log state in console"</button>
                        <p>"Current song: " {current_song_name}</p>
                    },
                )
            }
            Some(Err(e)) => EitherOf3::B(view! { <p>"Error activating player: " {e}</p> }),
            None => EitherOf3::C(view! { <p>"Activating player..."</p> }),
        }}
    }
}
