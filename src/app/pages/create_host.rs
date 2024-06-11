use leptos::{logging::*, *};
use leptos_router::*;

#[server]
async fn create_host(code: String, host_id: String) -> Result<(), ServerFnError> {
    use crate::general_types::AppState;
    use http::StatusCode;
    use serde::{Deserialize, Serialize};
    use sqlx::*;
    use std::collections::HashMap;

    let app_state = expect_context::<AppState>();
    let body = {
        let mut body = HashMap::new();
        body.insert("code", code.as_str());
        body.insert("redirect_uri", "http://localhost:3000/create-host");
        body.insert("grant_type", "authorization_code");
        body.insert("client_id", &app_state.spotify_credentials.id);
        body.insert("client_secret", &app_state.spotify_credentials.secret);
        body
    };
    let client = app_state.reqwest_client;
    let res = client
        .post("https://accounts.spotify.com/api/token")
        .form(&body)
        .send()
        .await?;

    #[derive(Serialize, Deserialize, Debug)]
    struct AccessToken {
        access_token: String,
        token_type: String,
        scope: String,
        expires_in: i64,
        refresh_token: String,
    }

    let res = match &res.status() {
        &StatusCode::OK | &StatusCode::CREATED => res.text().await?,
        _ => {
            log!("Error: {:?}", res);
            query("DELETE FROM hosts WHERE id = $1")
                .bind(host_id)
                .execute(&app_state.db.pool)
                .await?;
            return Err(ServerFnError::new(format!(
                "error while acquiring spotify token: {:#?}",
                res
            )));
        }
    };

    let token: AccessToken = serde_json::from_str(res.as_str())?;

    let now = time::OffsetDateTime::now_utc().unix_timestamp();
    let expires_at = now + token.expires_in;

    let query = query("UPDATE hosts SET access_token = ($1, $2, $3, $4) WHERE id = $5")
        .bind(token.access_token)
        .bind(expires_at)
        .bind(token.scope)
        .bind(token.refresh_token)
        .bind(host_id);

    query.execute(&app_state.db.pool).await?;

    Ok(())
}

#[component]
pub fn CreateHostPage() -> impl IntoView {
    let queries = use_query_map();
    let code = move || queries.with(|queries| queries.get("code").cloned());
    let host_id = move || queries.with(|queries| queries.get("state").cloned());

    let create_host_action = create_action(|input: &(String, String)| {
        use gloo::storage::{LocalStorage, Storage};

        let input = input.clone();
        async move { 
            let res=create_host(input.0.clone(), input.1.clone()).await;
            if res.is_ok()  {
                LocalStorage::set("host_id", input.1)?;
            }
            res
        }
    });

    let (feedback, set_feedback) = create_signal(String::from("creating host..."));

    create_effect(move |_| {
        if let (Some(code), Some(state)) = (code(), host_id()) {
            log!("Creating host with code: {} and state: {}", code, state);
            create_host_action.dispatch((code, state));
            create_host_action.pending();
        }
    });

    create_effect(move |_| {
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
