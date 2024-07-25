use general::Vote;
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
                    <Route path="/test" view=UserBartTest/>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
pub fn UserBartTest() -> impl IntoView {
    use crate::app::{components::UsersBar, general::User};
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
    let kick_user=|id| {
        log!("kicking user {}", id);
    };
    let kick_user=Callback::from(kick_user);

    view! {
        <UsersBar users=users kick_user=kick_user/>
        <button on:click=move |_| { set_users(None) }>"loading"</button>
    }
}
