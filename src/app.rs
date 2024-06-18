use leptos::*;
use leptos_meta::*;
use leptos_router::*;
pub mod pages;
pub mod components;

use components::error_template::*;
use serde::de::Error;

#[component]
fn Player() -> impl IntoView {
    use rust_spotify_web_playback_sdk::prelude::*;
    use std::rc::Rc;
    use std::cell::RefCell;

    let on_player_state_changed=Rc::new(RefCell::new(|state:Result<StateChange, impl std::error::Error>|{
        state.
    }));
    let connect=create_action(move|_:&()|{
       let on_player_state_changed=on_player_state_changed.clone();
       async {
        connect().await.unwrap();
        add_listener(Events::PlayerStateChanged(on_player_state_changed)).unwrap();
       }
    });

    let (ready, set_ready) = create_signal(false);
    let oauth= Rc::new(RefCell::new(|| "BQDJ8vSGHqeFrxQh-4BXOj-oIxqStlmmxpmVVv6U5zXOtuUwawcnqa99quXchaO9PAFrIfyRVfGCHVmbmLdtQNlfkbngEGIAC-ZnrA0fh1GqpKmrZ8Dd_MnJ3O1U_ZkbXpx1NiEqJQDxS9-pXBDv_NKWef9E0F_oouFXZXz9L45SgoAhqFS-y7ToLDcNIQHHW_lmHY68vK7RDabEtkdLvQIqnviZ".into()));
    let on_ready = Rc::new(RefCell::new(move || {
        connect.dispatch(());
        set_ready(true);
    }));
    init(oauth, on_ready, "kaki", 0.7, true);

    
    view! {
        <div>
            <h1>"Player ready:"{ready}</h1>
        </div>
    }
}


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
                    <Route path="/test" view=Player/>
                </Routes>
            </main>
        </Router>
    }
}

