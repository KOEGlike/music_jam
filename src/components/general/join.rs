use crate::components::modal::*;
use crate::components::user::SpinnyLoading;
use leptos::either::Either;
use leptos::{prelude::*, task::spawn_local};
use leptos_router::{hooks::use_navigate, NavigateOptions};

#[server]
async fn does_jam_exist(jam_code: String) -> Result<bool, ServerFnError> {
    use crate::model::{self, AppState};
    let app_state = expect_context::<AppState>();
    Ok(model::dose_jam_exist(&jam_code, &app_state.db.pool).await?)
}

#[derive(Debug, Clone)]
enum State {
    None,
    Loading,
    Error(String),
}

#[component]
pub fn JoinIsland() -> impl IntoView {
    let (jam_code, set_jam_code) = signal(String::from(""));
    let (state, set_state) = signal(State::None);

    let on_click = move |_| {
        set_state.set(State::Loading);
        spawn_local(async move {
            let res = does_jam_exist(jam_code.get_untracked()).await;

            match res {
                Err(e) => set_state.set(State::Error(format!(
                    "Error checking if jam exists: {:#?}",
                    e
                ))),
                Ok(false) => {
                    set_state.set(State::Error(String::from("Jam does not exist")));
                }
                Ok(true) => {
                    set_state.set(State::None);
                    if !jam_code.with_untracked(String::is_empty) {
                        let navigate = use_navigate();
                        navigate(
                            format!("/create-user/{}", jam_code.get_untracked()).as_str(),
                            NavigateOptions::default(),
                        );
                    }
                }
            }
        });
    };

    view! {
        <div class="join-island">
            <Modal visible=Signal::derive(move || {
                matches!(state.get(), State::Error(_))
            })>

                {move || {
                    match state.get() {
                        State::Error(err) => Either::Left(view! { <p>{err}</p> }),
                        _ => Either::Right(()),
                    }
                }}
                <button on:click=move |_| {
                    set_state.set(State::None);
                }>"Close"</button>

            </Modal>

            <div class="jam-id-input">
                <label for="join-text-input">"Jam Code"</label>
                <input
                    type="text"
                    maxlength=6
                    prop:value=jam_code
                    on:input=move |ev| set_jam_code.set(event_target_value(&ev))
                    placeholder="ex. 786908"
                    class="text-input"
                    id="join-text-input"
                    pattern="{6}"
                />
            </div>
            <button on:click=on_click class="join-button">
                {move || match state.get() {
                    State::Loading => Either::Left(view! { <SpinnyLoading /> }),
                    _ => Either::Right(view! { "Join" }),
                }}
            </button>
        </div>
    }
}
