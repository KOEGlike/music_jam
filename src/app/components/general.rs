
use leptos::{logging::log, prelude::*, *};


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
async fn redirect_to_spotify_oauth() -> Result<(), ServerFnError> {
    use leptos_axum::redirect;
    use crate::AppState;
    let spotify_id=expect_context::<AppState>().spotify_id;
    redirect(
        format!(
            "https://accounts.spotify.com/authorize?response_type=code+client_id={}+scope={}+redirect_uri={}+state{}"
            ,spotify_id
            ,"user-read-playback-state user-modify-playback-state user-read-currently-playing"
            ,"/"
            ,cuid2::create_id()
        ).as_str()
    );
    Ok(())
}



#[server]
async fn create_jam() -> Result<(), ServerFnError> {
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
