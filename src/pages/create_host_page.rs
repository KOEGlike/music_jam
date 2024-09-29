use leptos::{logging::*, prelude::*, *};
use leptos_router::{hooks::*, *};

#[server]
async fn create_host(code: String, host_id: String) -> Result<(), ServerFnError> {
    use crate::model::functions;
    use crate::model::AppState;
    let app_state = expect_context::<AppState>();

    if let Err(e) = functions::create_host(
        code,
        host_id,
        &app_state.spotify_credentials,
        &app_state.reqwest_client,
        &app_state.db.pool,
        &format!("{}/create-host", app_state.site_url),
    )
    .await
    {
        return Err(ServerFnError::ServerError(format!("{:#?}", e)));
    }

    Ok(())
}

#[component]
pub fn CreateHostPage() -> impl IntoView {
    let queries = use_query_map();
    let code = move || queries.with(|queries| queries.get("code"));
    let host_id = move || queries.with(|queries| queries.get("state"));

    let create_host_action = Action::new(|input: &(String, String)| {
        use gloo::storage::{LocalStorage, Storage};

        let input = input.clone();
        async move {
            let res = create_host(input.0.clone(), input.1.clone()).await;
            if res.is_ok() {
                LocalStorage::set("host_id", input.1)?;
            }
            res
        }
    });

    let (feedback, set_feedback) = signal(String::from("Creating host..."));

    Effect::new(move |_| {
        if let (Some(code), Some(state)) = (code(), host_id()) {
            log!("Creating host with code: {} and state: {}", code, state);
            create_host_action.dispatch((code, state));
            create_host_action.pending();
        } else {
            set_feedback("Error creating host: missing code or state".to_string());
        }
    });

    Effect::new(move |_| {
        if let Some(res) = create_host_action.value().get() {
            match res {
                Ok(_) => {
                    set_feedback("Host created successfully!".to_string());
                }
                Err(err) => {
                    set_feedback(format!("Error creating host: {:#?}", err));
                }
            }
            let timer = gloo::timers::callback::Timeout::new(2000, || {
                let navigate = use_navigate();
                navigate("/", NavigateOptions::default());
            });

            timer.forget();
        }
    });

    view! {
        <div id="create-host=page">
            <div id="create-host-island">{feedback}</div>
        </div>
    }
}
