use crate::general;
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
pub mod components;
pub mod pages;


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
                    <Route path="/test-bar" view=UserBartTest/>
                    <Route path="/test-share" view=ShareTest/>
                    <Route path="/test-search" view=SearchTest/>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
pub fn UserBartTest() -> impl IntoView {
    use crate::{general::User,app::components::UsersBar};
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

#[component]
fn ShareTest() -> impl IntoView {
    use leptos::logging::*;
    use crate::app::components::Share;
    view! {
        <Share jam_id="5Y8FXC"/>
    }
}

#[component]
fn SearchTest() -> impl IntoView {
    use leptos::logging::*;
    use crate::app::components::Search;

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
        votes: general::Vote{votes: 0, have_you_voted:None},
    };

    let songs = {
        let mut songs = Vec::new();
        for _ in 0..10 {
            songs.push(song.clone());
        }
        songs
    };
    let (songs, _) = create_signal(Some(songs));
    let search=move|id|log!("search with id:{}", id);
    let search=Callback::from(search);

    let add_song=move|id|log!("add with id:{}", id);
    let add_song=Callback::from(add_song);
    view! {
        <Search search_result=songs search add_song/>
    }
}
