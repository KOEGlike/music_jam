
use leptos::{logging::log, prelude::*, *};
use cuid2::*;
use sqlx::query;
use crate::AppState;
use rspotify::{AuthCodeSpotify, Credentials, OAuth, Token};

#[component]
pub fn JoinIsland() -> impl IntoView {
    let (jam_code, set_jam_code) = create_signal(String::new());
    let on_click = move |_| {
        log!("Joining jam code: {}", jam_code.get());
    };

    view! {
        <div class="big-space-island">
            <div id="join-text">"image goes here"</div>
            <div class="input-with-label">
                <label for="join-text-input">"Jam Code"</label>
                <input
                    type="text"
                    maxlength=6
                    prop:value=jam_code
                    on:input=move |ev| set_jam_code(event_target_value(&ev))
                    placeholder="ex. 786908"
                    class="text-input"
                    id="join-text-input"/>
            </div>
            <button on:click=on_click class="button">"Join"</button>
        </div>
    }
}

#[server]
async fn create_host() -> Result<(), ServerFnError> {
    let idk=AuthCodeSpotify::from_token(Token{})
    log!("Creating host"); 
    Ok(())
}

#[server]
async fn create_jam(jam_name: String) -> Result<(String), ServerFnError> {
    let db_pool = expect_context::<AppState>().db_pool;
    let query= sqlx::query!("INSERT INTO "hosts" ("id", "access_token") VALUES ('your_host_id', 'your_access_token');");
    Ok(())
}

#[component]
pub fn CreateIsland() -> impl IntoView {
    let (name, set_name) = create_signal(String::new());

    let on_click = move |_| {
        log!("Creating island");
    };

    view! {
        <div class="big-space-island" id="create-island">
            <div id="join-text">"image goes here"</div>
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
            
            <button on:click=on_click class="button">"Create"</button>
        </div>
    }
}
