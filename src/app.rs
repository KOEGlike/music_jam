use crate::model;
use crate::model::real_time::SearchResult;
use leptos::{prelude::*, task::spawn_local};
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
                <link rel="apple-touch-icon" sizes="180x180" href="/favicon/apple-touch-icon.png" />
                <link rel="icon" type="image/png" sizes="32x32" href="/favicon/favicon-32x32.png" />
                <link rel="icon" type="image/png" sizes="16x16" href="/favicon/favicon-16x16.png" />
                <link rel="manifest" href="/favicon/site.webmanifest" />

                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <link rel="stylesheet" href="/pkg/music_jam.css" />

                <title>"Music Jam"</title>
                <meta
                    name="description"
                    content="This web app is for solving the problem of bad music at parties. The users can submit songs to a queue and like those songs, the song with the most likes gets played using a Spotify integration."
                />

                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    let (is_loading, set_is_loading) = signal(false);
    Effect::new(move |_| {
        set_is_loading.set(false);
    });
    view! {
        <Router>
            <main style:display=move || if is_loading.get() { "none" } else { "inline-block" }>
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=path!("/") view=pages::HomePage />
                    <Route path=path!("/create-host") view=pages::CreateHostPage />
                    <Route path=path!("/create-user/:id") view=pages::CreateUserPage />
                    <Route path=path!("/jam/host/:id") view=pages::HostPage />
                    <Route path=path!("/jam/:id") view=pages::UserPage />
                    <Route path=path!("/test-bar") view=UserBartTest />
                    <Route path=path!("/test-share") view=ShareTest />
                    <Route path=path!("/test-search") view=SearchTest />
                    <Route path=path!("/test-user-player") view=UserPlayerTest />
                    <Route path=path!("/test-sdk") view=SDKTest />
                    <Route path=path!("/test-song-list") view=SongListTest />
                </Routes>
            </main>
            <div
                class="loading-indicator"
                style:display=move || if is_loading.get() { "inline-block" } else { "none" }
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
    let (songs, _) = signal(Some(songs));
    let (votes, _) = signal(model::Votes::new());
    let max_song_count = Signal::derive(|| 10);
    let song_list_action = SongListAction::Vote {
        add_vote: Callback::new(|id| log!("add vote with id:{}", id)),
        remove_vote: Callback::new(|id| log!("remove vote with id:{}", id)),
        remove_song: Callback::new(|id| log!("remove song with id:{}", id)),
    };
    view! { <SongList songs votes max_song_count song_list_action /> }
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
        <UsersBar close users kick_user />
        <button on:click=move |_| { set_users.set(None) }>"loading"</button>
    }
}

#[component]
fn ShareTest() -> impl IntoView {
    use crate::components::Share;
    let jam_id = Signal::derive(|| "5Y8FXC".to_owned());
    view! { <Share jam_id /> }
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
        set_search_result.set(Some(SearchResult {
            songs: songs.clone(),
            search_id: id.1,
        }));
        log!("search with id:{}", id.0)
    };
    let search = Callback::new(search);

    let add_song = move |id| log!("add with id:{}", id);
    let add_song = Callback::new(add_song);
    let loaded = Signal::derive(|| true);

    view! { <Search search_result search add_song loaded /> }
}

#[component]
fn UserPlayerTest() -> impl IntoView {
    use crate::components::general::Player;

    let (current_song, _) = signal(Some(model::Song {
        id: Some("lol".to_string()),
        spotify_id: "lol".to_string(),
        user_id: None,
        name: "Yesterdayyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyy".to_string(),
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

    view! { <Player position current_song=current_song /> }
}

#[component]
fn SDKTest() -> impl IntoView {
    use leptos::either::EitherOf3;
    use leptos::logging::*;
    use rust_spotify_web_playback_sdk::prelude as sp;

    let (current_song_name, set_current_song_name) = signal(String::new());

    let token = "BQA7ffYb5jY2Ahm1zInbamFfF02shMeoCxX8ccqurw_hlkyzGMtWqL9vFWsef9lq8ahTx4whfzjcodUaGnOoZ4o3954CDkNSkoQ-Nk1zLi9ky7ymq5ErMOza98gH8Cv6dzfqm2Rwr2gbo6P-JY4mSHhzSOEPUxMkJ6afUrIdRoJRJ4NM6buVwZxuBv46AC8vUacBvhkBfJUzt6WvsI9A5g16RcvIPqlHpqS8";

    let connect = || {
        spawn_local(async {
            match sp::connect().await {
                Ok(_) => log!("connected"),
                Err(e) => log!("error {:?}", e),
            };
        });
    };

    Effect::new(move |_| {
        sp::init(
            || {
                log!("oauth was called");
                token.to_string()
            },
            move || {
                log!("ready");

                let res =
                    sp::add_listener!("player_state_changed", move |state: sp::StateChange| {
                        log!("state changed, {}", state.track_window.current_track.name);
                        set_current_song_name.set(state.track_window.current_track.name);
                    });
                connect();

                log!("res ");
                log!("{:?}", res);
            },
            "example player",
            1.0,
            false,
        );
    });

    let get_state = || {
        spawn_local(async {
            let state = sp::get_current_state().await.unwrap();
            log!("{:#?}", state);
        });
    };

    let activate_player: Action<_, _> =
        Action::new_unsync(|_| async { sp::activate_element().await });

    view! {
        <h1>"Welcome to Leptos"</h1>
        <button on:click=move |_| {
            activate_player.dispatch(());
        }>"activate player"</button>

        {move || match activate_player.value().get() {
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
