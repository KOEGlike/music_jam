use crate::app::components::*;

use leptos::*;
use leptos_router::*;


#[server]
async fn create_host(code: String, host_id: String) -> Result<(), ServerFnError> {
    use serde::{Deserialize, Serialize};
    use sqlx::*;
    use crate::AppState;
    let app_state = expect_context::<AppState>();

    let body = &[
        ("code", code.as_str()),
        ("redirect_uri", "/"),
        ("grant_type", "authorization_code"),
    ];

    let client = app_state.reqwest_client;
    let res = client
        .post("https://accounts.spotify.com/api/token")
        .header("Content-Type", " application/x-www-form-urlencoded")
        .header(
            "Authorization",
            format!("{}:{}", app_state.spotify_id, app_state.spotify_secret),
        )
        .form(body)
        .send()
        .await?;

    #[derive(Serialize, Deserialize)]
    struct AccessToken {
        access_token: String,
        token_type: String,
        scope: String,
        expires_in: i64,
        refresh_token: String,
    }
    let res = match res.error_for_status() {
        Ok(res) => res.text().await?,
        Err(err) => {
            let err = "Access token couldn't be acquired: ".to_string() + err.to_string().as_str();
            return Err(ServerFnError::new(err));
        }
    };

    let token = serde_urlencoded::from_str::<AccessToken>(res.as_str())?;

    let now = ::time::OffsetDateTime::now_utc().unix_timestamp();
    let expires_at = now + token.expires_in;

    let query= query("INSERT INTO \"hosts\"(\"id\", \"access_token\") VALUES ($1, ROW($2, $3, $4, $5)::access_token)")
        .bind(host_id)
        .bind(token.access_token)
        .bind(expires_at)
        .bind(token.scope)
        .bind(token.refresh_token);

    let pool = app_state.db_pool;
    pool.acquire().await?.execute(query).await?;

    Ok(())
}

#[component]
pub fn HomePage() -> impl IntoView {

    let queries = use_query_map();
    let code = move || {
        queries.with(|queries| {
            queries.get("code").cloned()
        })
    };

    view! {
        <div id="home-page">
            <JoinIsland/>
            <CreateIsland/>
        </div>
    }
}
