use leptos::*;
use leptos_meta::*;
use leptos_router::*;
pub mod components;
pub mod pages;
#[cfg(feature = "ssr")]
pub mod general_functions;
#[cfg(feature = "ssr")]
pub mod socket;

use components::error_template::*;

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();
    view! {


        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/music_jam.css"/>

        // sets the document title
        <Title text="Welcome to Leptos"/>

        // content for this welcome page
        <Router fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! {
                <ErrorTemplate outside_errors/>
            }
            .into_view()
        }>
            <main>
                <Routes>
                    <Route path="/" view=pages::HomePage/>
                    <Route path="/create-host" view=pages::CreateHostPage/>
                    <Route path="/test" view=TestSocketRead/>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn TestSocketRead() -> impl IntoView {
    use crate::general_types::*;
    use leptos_use::{core::ConnectionReadyState, use_websocket, UseWebsocketReturn};
    let id = "123456789012345678901234";

    let UseWebsocketReturn {
        ready_state,
        message_bytes,
        send_bytes,
        open,
        message,
        ..
    } = use_websocket(&format!("ws://localhost:3000/socket?id={}", id));

    let update = move || match message_bytes() {
        Some(m) => rmp_serde::from_slice::<real_time::Update>(&m).unwrap(),
        None => real_time::Update::Error(real_time::Error::Database(
            "idk this is on the client side so it should never happen".to_string(),
        )),
    };

    let send = move || {
        let message = real_time::Request::AddSong {
            song_id: "song_id".to_string(),
        };
        let message = rmp_serde::to_vec(&message).unwrap();
        send_bytes(message);
    };

    create_effect(move |_| {
        let message = match ready_state() {
            ConnectionReadyState::Connecting => "Connecting",
            ConnectionReadyState::Open => "Open",
            ConnectionReadyState::Closing => "Closing",
            ConnectionReadyState::Closed => "Closed",
        };
        leptos::logging::log!("{message}");
    });

    create_effect(move |_| {
        leptos::logging::log!("{:?}",message());
    });

    create_effect(move |_| {
        leptos::logging::log!("{:?}", update());
    });

    view! {
        <button on:click= move |_|open() >"Open"</button>
        <br/>
        <button on:click= move |_|send() >"Send"</button>
        <br/>
        {move || match ready_state() {
            ConnectionReadyState::Connecting => "Connecting",
            ConnectionReadyState::Open => "Open",
            ConnectionReadyState::Closing => "Closing",
            ConnectionReadyState::Closed => "Closed",
        }}
        <br/>
        {move || format!("{:#?}", update())}


    }
}
