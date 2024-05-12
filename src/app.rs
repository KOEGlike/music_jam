use crate::error_template::{AppError, ErrorTemplate};
use leptos::{logging::log, *};
use leptos_meta::*;
use leptos_router::*;

use wasm_bindgen::{closure::Closure, JsValue};

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
                    <Route path="" view=HomePage/>
                </Routes>
            </main>
        </Router>
    }
}

use rust_spotify_web_playback_sdk as sp;
#[component]
fn HomePage() -> impl IntoView {
    let (is_sp_ready, set_is_sp_ready) = create_signal(false);
   /* if cfg!(any(target_arch = "wasm32", target_arch = "wasm64")) {
        log!("wasm");
        let token="BQDK6xOHstpoxt_dIh6y7dqcog-QNHe-prqdUhK784N1EsmQ2YSC17vwCX8mbFsVepNLjCWSVCmAq_lZxrb0e_PJ-QlcKXzOR9_nPFQ_JbvCOET3lsq-J-4JAQ-zr5ls8Q30rlBTzxrAcvRZq8IpjyLUMKdGEM7nFQt0Nf8Bfjf0NuUiNeSrd5awupGrOKFp4DtXSEvkyH-vUw0Iv8TpqdxvLVk2";
        let oauth_cb = || {
            log!("oauth was called");
            token.to_string()
        };
        let oauth_cb = Closure::new(oauth_cb);
        let update_signal = move || {
            set_is_sp_ready(true);
        };
        let on_ready = Closure::new(update_signal);

        create_effect(move |_| {
            sp::init(
                &oauth_cb,
                &on_ready,
                "example player".to_string(),
                1.0,
                false,
            );
        });
    }*/

    let connect = create_action(|_| async {
        match sp::connect().await {
            Ok(_) => log!("connected"),
            Err(e) => log!("error {:?}", e.as_string()),
        };
    });

    let (current_song_name, set_current_song_name) = create_signal(String::new());

    if cfg!(any(target_arch = "wasm32", target_arch = "wasm64")) {
        let cb = Closure::new(move |jsv: JsValue| {
            let state: sp::structs::state_change::StateChange = sp::structs::from_js(jsv).unwrap();
            log!("state changed, {}", state.track_window.current_track.name);
            set_current_song_name(state.track_window.current_track.name);
        });
        create_effect(move |_| {
            if is_sp_ready() {
                log!("ready");
                connect.dispatch(());
                sp::add_listener("player_state_changed", &cb);
            }
        });
    }

    view! {
        <h1>"Spotify player"</h1>
        <button on:click=move |_|connect.dispatch(())>"Connect"</button>
        <p>"Current song: " {move || current_song_name()}</p>
    }
}
