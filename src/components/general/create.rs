use crate::components::Modal;
use crate::general::*;
use leptos::{logging::log, prelude::*, *};
use leptos_router::*;

#[server]
async fn redirect_to_spotify_oauth() -> Result<(), ServerFnError> {
    use crate::general::AppState;
    use leptos_axum::redirect;
    use sqlx::*;
    let app_state = expect_context::<AppState>();

    let host_id = cuid2::create_id();
    let query = query!("INSERT INTO hosts(id) VALUES ($1)", &host_id);
    let pool = app_state.db.pool;
    query.execute(&pool).await?;
    redirect(
        format!(
            "https://accounts.spotify.com/authorize?response_type=code&client_id={}&scope={}&redirect_uri={}&state={}&show_dialog=false"
            ,app_state.spotify_credentials.id
            ,"user-read-playback-state user-modify-playback-state user-read-currently-playing streaming user-read-private user-read-email"
            ,"http://localhost:3000/create-host"
            ,host_id
        ).as_str()
    );
    Ok(())
}

#[server]
async fn create_jam(
    name: String,
    host_id: String,
    max_song_count: i16,
) -> Result<JamId, ServerFnError> {
    use crate::general::{AppState, create_jam, Error};
    let app_state = expect_context::<AppState>();

    

     match create_jam(&name, &host_id, max_song_count, &app_state.db.pool).await {
        Ok(jam_id) => Ok(jam_id),
        Err(Error::HostAlreadyInJam { jam_id }) => Ok(jam_id),
        Err(e) => Err(ServerFnError::Request(format!("Error creating jam: {:#?}", e))),
     }
}

#[component]
pub fn CreateIsland() -> impl IntoView {
    use gloo::storage::{errors::StorageError, LocalStorage, Storage};

    let (name, set_name) = create_signal(String::new());
    let (max_song_count, set_max_song_count) = create_signal::<i16>(1);
    let (error_message, set_error_message) =
        create_signal(String::from("there is no error lol, this is a bug"));
    let (show_dialog, set_show_dialog) = create_signal(false);

    let redirect_to_oauth = create_action(move |_| {
        if let Err(err) = LocalStorage::set("jam_name", name()) {
            log!("Error setting jam name to local storage: {}", err);
        }

        async move {
            match redirect_to_spotify_oauth().await {
                Ok(_) => log!("Redirected to Spotify OAuth"),
                Err(e) => log!("Error redirecting to Spotify OAuth: {}", e),
            };
        }
    });

    let create = create_action(move |_: &()| {
        let name = name();
        let max_song_count = max_song_count();
        async move {
            match LocalStorage::get::<String>("host_id") {
                Ok(host_id) => match create_jam(name, host_id, max_song_count).await {
                    Ok(jam_id) => {
                        let navigate = use_navigate();
                        navigate(
                            format!("/jam/host/{}", jam_id).as_str(),
                            NavigateOptions::default(),
                        );
                    }
                    Err(e) => {
                        set_error_message(format!("Error creating jam: {}", e));
                        set_show_dialog(true);
                    }
                },
                Err(StorageError::KeyNotFound(_)) => redirect_to_oauth.dispatch(()),
                Err(e) => log!("Error getting host id from local storage: {}", e),
            };
        }
    });

    create_effect(move |_| {
        let name: Result<String, StorageError> = LocalStorage::get("jam_name");
        match name {
            Ok(name) => set_name(name),
            Err(e) => log!("Error getting jam name from local storage: {}", e),
        }
    });

    view! {
        <Modal visible=show_dialog>
            {error_message} <button on:click=move |_| set_show_dialog(false) class="button">
                "Close"
            </button>
        </Modal>
        <div class="big-space-island" id="create-island">
            <div id="join-text">"image goes here"</div>
            <div class="input-with-label">
                <div class="input-with-label">
                    <label for="create-jam-name">"Jam Name"</label>
                    <input
                        type="text"
                        prop:value=name
                        on:input=move |ev| set_name(event_target_value(&ev))
                        placeholder="ex. My Jam"
                        class="text-input"
                        id="create-jam-name"
                    />
                </div>
                <div class="input-with-label">
                    <label for="create-jam-max-songs">"Max Songs"</label>
                    <input
                        type="number"
                        prop:value=max_song_count
                        on:input=move |ev| set_max_song_count(
                            event_target_value(&ev).parse().unwrap_or(0),
                        )

                        placeholder="ex. 10"
                        class="text-input"
                        id="create-jam-max-songs"
                        min=1
                    />
                </div>
            </div>

            <button on:click=move |_| create.dispatch(()) class="button">
                "Create"
            </button>
        </div>
    }
}
