use crate::components::Modal;
use crate::model::*;
use leptos::task::spawn_local;
use leptos::{either::*, logging::log, prelude::*};
use leptos_router::{hooks::use_navigate, *};

#[server]
async fn redirect_to_spotify_oauth() -> Result<(), ServerFnError> {
    use crate::model::AppState;
    use leptos_axum::*;
    use sqlx::*;
    let app_state = expect_context::<AppState>();

    let host_id = cuid2::create_id();
    let query = query!("INSERT INTO hosts(id) VALUES ($1)", &host_id);
    let pool = app_state.db.pool;
    query.execute(&pool).await?;
    redirect(
        format!(
            "https://accounts.spotify.com/authorize?response_type=code&client_id={}&scope={}&redirect_uri={}/create-host&state={}&show_dialog=false"
            ,app_state.spotify_credentials.id
            ,"user-read-playback-state user-modify-playback-state user-read-currently-playing streaming user-read-private user-read-email"
            ,app_state.site_url
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
    use crate::model::{create_jam, set_current_song, AppState, Error};
    let app_state = expect_context::<AppState>();
    let mut transaction = app_state.db.pool.begin().await?;
    let credentials = app_state.spotify_credentials;

    let res = match create_jam(
        &name,
        &host_id,
        max_song_count,
        &mut transaction,
        credentials.clone(),
    )
    .await
    {
        Ok(jam_id) => {
            let song =
                match get_current_song_from_player(&jam_id, &mut transaction, credentials.clone())
                    .await
                {
                    Ok(song) => match song {
                        Some(song) => song,
                        None => search(
                            "Never gonna give you up",
                            &mut transaction,
                            &jam_id,
                            credentials.clone(),
                        )
                        .await?
                        .remove(0),
                    },
                    Err(e) => return Err(e.into()),
                };
            if let Err(e) = set_current_song(&song, &jam_id, &mut *transaction).await {
                return Err(ServerFnError::Request(format!(
                    "Error setting current song: {:?}",
                    e
                )));
            }
            Ok(jam_id)
        }
        Err(Error::HostAlreadyInJam { jam_id }) => Ok(jam_id),
        Err(e) => Err(ServerFnError::Request(format!(
            "Error creating jam: {:#?}",
            e
        ))),
    };
    transaction.commit().await?;
    res
}

#[component]
pub fn CreateIsland() -> impl IntoView {
    use gloo::storage::{errors::StorageError, LocalStorage, Storage};

    let (host_id, set_host_id) = signal(None::<String>);

    let (name, set_name) = signal(String::from(""));
    let (max_song_count, set_max_song_count) = signal::<i16>(1);

    let (error_message, set_error_message) =
        signal(String::from("there is no error lol, this is a bug"));
    let (show_dialog, set_show_dialog) = signal(false);

    Effect::new(move |_| match LocalStorage::get::<String>("host_id") {
        Ok(id) => set_host_id(Some(id)),
        Err(StorageError::KeyNotFound(_)) => (),
        Err(e) => {
            set_error_message(format!("Error getting host id from local storage: {}", e));
            set_show_dialog(true);
        }
    });

    let redirect_to_oauth = move || {
        spawn_local(async move {
            match redirect_to_spotify_oauth().await {
                Ok(_) => log!("Redirected to Spotify OAuth"),
                Err(e) => log!("Error redirecting to Spotify OAuth: {}", e),
            };
        });
    };

    let create = Action::new(move |_: &()| {
        let name = name.get_untracked();
        let max_song_count = max_song_count();
        async move {
            match host_id.get_untracked() {
                Some(host_id) => {
                    if !name.is_empty() {
                        match create_jam(name, host_id, max_song_count).await {
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
                        }
                    }
                }
                None => {
                    set_error_message("this is a bug, how did this button get pressed".to_string());
                    set_show_dialog(true);
                }
            };
        }
    });

    view! {
        <Modal visible=show_dialog>
            {error_message} <button on:click=move |_| set_show_dialog(false) class="button">
                "Close"
            </button>
        </Modal>
        <div class="big-space-island" id="create-island">
            {move || {
                if host_id.with(Option::is_some) {
                    Either::Left(
                        view! {
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
                                        class:glass-element-err=move || name.with(String::is_empty)
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

                            <button
                                on:click=move |_| {
                                    create.dispatch(());
                                }

                                class="button"
                            >
                                "Create"
                            </button>
                        },
                    )
                } else {
                    Either::Right(
                        view! {
                            <div id="to-create-jam">
                                "You need to connect your Spotify account to create a jam"
                            </div>
                            <button on:click=move |_| redirect_to_oauth() class="connect-spotify">
                                <div>"Connect Spotify"</div>
                                <svg xmlns="http://www.w3.org/2000/svg" width="64" height="64">
                                    <path d="M32 0C14.3 0 0 14.337 0 32c0 17.7 14.337 32 32 32 17.7 0 32-14.337 32-32S49.663 0 32 0zm14.68 46.184c-.573.956-1.797 1.223-2.753.65-7.532-4.588-16.975-5.62-28.14-3.097-1.07.23-2.14-.42-2.37-1.49s.42-2.14 1.49-2.37c12.196-2.79 22.67-1.606 31.082 3.556a2 2 0 0 1 .688 2.753zm3.9-8.717c-.726 1.185-2.256 1.53-3.44.84-8.602-5.276-21.716-6.805-31.885-3.747-1.338.382-2.714-.344-3.097-1.644-.382-1.338.344-2.714 1.682-3.097 11.622-3.517 26.074-1.835 35.976 4.244 1.147.688 1.49 2.217.765 3.403zm.344-9.1c-10.323-6.117-27.336-6.69-37.2-3.708-1.568.497-3.25-.42-3.747-1.988s.42-3.25 1.988-3.747c11.317-3.44 30.127-2.753 41.98 4.282 1.415.84 1.873 2.676 1.032 4.09-.765 1.453-2.638 1.912-4.053 1.07z"></path>
                                </svg>
                            </button>
                        },
                    )
                }
            }}

        </div>
    }
}
