use crate::general::{functions::notify_all, types::*};
use leptos::logging::*;

use super::notify;

pub async fn create_host(
    code: String,
    host_id: String,
    spotify_credentials: &SpotifyCredentials,
    reqwest_client: &reqwest::Client,
    pool: &sqlx::PgPool,
) -> Result<(), Error> {
    use http::StatusCode;
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    let body = {
        let mut body = HashMap::new();
        body.insert("code", code.as_str());
        body.insert("redirect_uri", "http://localhost:3000/create-host");
        body.insert("grant_type", "authorization_code");
        body.insert("client_id", &spotify_credentials.id);
        body.insert("client_secret", &spotify_credentials.secret);
        body
    };
    let client = reqwest_client;
    let res = match client
        .post("https://accounts.spotify.com/api/token")
        .form(&body)
        .send()
        .await
    {
        Ok(res) => res,
        Err(e) => {
            return Err(Error::Spotify(format!(
                "error while acquiring temp spotify token: {:#?}",
                e
            )))
        }
    };

    #[derive(Serialize, Deserialize, Debug)]
    struct AccessToken {
        access_token: String,
        token_type: String,
        scope: String,
        expires_in: i64,
        refresh_token: String,
    }

    let res = match &res.status() {
        &StatusCode::OK | &StatusCode::CREATED => res.text().await,
        _ => {
            log!("Error: {:?}", res);
            sqlx::query!("DELETE FROM hosts WHERE id = $1", host_id)
                .execute(pool)
                .await?;
            return Err(Error::Database(format!(
                "error while acquiring spotify token, spotify returned not ok response code: {:#?}",
                res
            )));
        }
    };

    let res = match res {
        Ok(res) => res,
        Err(e) => {
            return Err(Error::Decode(format!(
                "error while getting text from spotify response: {:#?}",
                e
            )))
        }
    };

    let token: AccessToken = match serde_json::from_str(res.as_str()) {
        Ok(token) => token,
        Err(e) => {
            return Err(Error::Decode(format!(
                "error while deserializing spotify response spotify token: {:#?}",
                e
            )))
        }
    };

    let now = chrono::Utc::now().timestamp();
    let expires_at = now + token.expires_in;

    let access_token_id = cuid2::create_id();
    sqlx::query!(
        "INSERT INTO access_tokens 
            (access_token, expires_at, scope, refresh_token,id, host_id) 
        VALUES 
            ($1, $2, $3, $4,$5,$6)",
        token.access_token,
        expires_at,
        token.scope,
        token.refresh_token,
        access_token_id,
        host_id
    ).execute(pool).await?;

    Ok(())
}

pub async fn delete_jam(jam_id: &str, pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    sqlx::query!("DELETE FROM jams WHERE id = $1", jam_id)
        .execute(pool)
        .await?;
    notify(real_time::Channels::Ended, jam_id, pool).await?;
    Ok(())
}

pub async fn create_jam(
    name: String,
    host_id: String,
    max_song_count: i16,
    pool: &sqlx::PgPool,
) -> Result<JamId, Error> {
    let jam_id = cuid2::CuidConstructor::new().with_length(6).create_id();

    sqlx::query!(
        "INSERT INTO jams (id, max_song_count, host_id, name) VALUES ($1, $2, $3, $4)",
        &jam_id,
        &max_song_count,
        &host_id,
        &name
    )
    .execute(pool)
    .await?;

    tokio::spawn(occasional_notify(pool.clone(), jam_id.clone()));

    Ok(jam_id)
}

async fn occasional_notify(pool: sqlx::PgPool, jam_id: String) -> Result<(), Error> {
    use std::time::Duration;
    loop {
        if let Err(e) = notify_all(&jam_id, &pool).await {
            eprintln!("Error notifying all, in occasional notify: {:?}", e);
        };
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}
