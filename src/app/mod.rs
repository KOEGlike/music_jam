use leptos::*;
use leptos_meta::*;
use leptos_router::*;
pub mod components;
pub mod general;
pub mod pages;

#[cfg(feature = "ssr")]
pub mod socket;

use components::error_template::*;

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
                    <Route path="/test" view=SongListTest/>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
pub fn SongListTest() -> impl IntoView {
    use components::{SongAction, SongList};
    let cb = Callback::new(move |id| {
        leptos::logging::log!("Add song with id: {}", id);
    });
    let song_action = SongAction::Add(cb);

    let song = general::Song {
        id: "lol".to_string(),
        user_id: None,
        name: "Yesterday".to_string(),
        artists: vec!["Beatles".to_string()],
        album: "Help!".to_string(),
        duration: 240,
        image: general::Image {
            height: Some(64),
            url: "https://i.scdn.co/image/ab67616d0000b273e3e3b64cea45265469d4cafa".to_string(),
            width: Some(64),
        },
        votes: 2,
    };

    let songs = {
        let mut songs = Vec::new();
        for _ in 0..10 {
            songs.push(song.clone());
        }
        songs
    };
    let (songs, _) = create_signal(songs);
    let (votes, _) = create_signal(general::Votes::new());

    view! { <SongList songs=songs votes=votes song_action=song_action request_update=move||{leptos::logging::log!("requested update.....")}/> }
}