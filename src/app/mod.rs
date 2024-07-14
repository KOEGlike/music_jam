use leptos::*;
use leptos_meta::*;
use leptos_router::*;
pub mod components;
pub mod general;
pub mod pages;

#[cfg(feature = "ssr")]
pub mod socket;

use components::{error_template::*, song};

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
                    <Route path="/test" view=Test/>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
pub fn Test() -> impl IntoView {
    use components::{Song, SongAction};
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
            url: "https://i.scdn.co/image/ab67616d00004851099b70ad78a864219e894dca".to_string(),
            width: Some(64),
        },
        votes: 2,
    };

    view! { <Song song=song song_action=song_action/> }
}
