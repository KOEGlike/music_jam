use super::{handle_error, IdType};
use crate::general_types::*;
use axum::extract::ws;
use sqlx::postgres::PgListener;
use tokio::sync::mpsc;

pub async fn write(sender: mpsc::Sender<ws::Message>, id: IdType, app_state: AppState) {
    let pool = app_state.db.pool;

    tokio::spawn(listen_songs(
        pool.clone(),
        id.jam_id().into(),
        sender.clone(),
    ));
}

async fn listen_songs(
    pool: sqlx::PgPool,
    jam_id: String,
    sender: mpsc::Sender<ws::Message>,
) -> Result<(), real_time::Error> {
    let mut listener = create_listener(&pool, &jam_id, "songs").await?;

    while let Ok(m) = listener.try_recv().await {
        if m.is_none() {
            let error = real_time::Error::Database("pool disconnected reconnecting...".to_string());
            handle_error(error, false, &sender).await;
            continue;
        }

        let songs = sqlx::query_as!(
            Song,
            "SELECT s.*, COUNT(v.id) AS votes
FROM songs s
JOIN users u ON s.user_id = u.id
LEFT JOIN votes v ON s.id = v.song_id
WHERE u.jam_id = $1
GROUP BY s.id",
            jam_id
        )
        .fetch_all(&pool)
        .await;

        let songs = match songs {
            Ok(songs) => songs,
            Err(e) => {
                let error = real_time::Error::Database(e.to_string());
                handle_error(error, false, &sender).await;
                continue;
            }
        };

        let bin = rmp_serde::to_vec(&real_time::Update::Songs(songs)).unwrap();
    }

    Ok(())
}

async fn get_access_token(
    pool: &sqlx::PgPool,
    jam_id: &str,
) -> Result<rspotify::Token, real_time::Error> {
    struct AccessTokenDb {
        pub refresh_token: String,
        pub access_token: String,
        pub expires_at: i64,
        pub scope: String,
        pub id: String,
    }

    let token = sqlx::query_as!(
        AccessTokenDb,
        "SELECT * FROM access_tokens WHERE id=(SELECT access_token FROM hosts WHERE id=(SELECT host_id FROM jams WHERE id=$1))",
        jam_id
    )
    .fetch_one(pool)
    .await?;

    let expires_at = chrono::DateTime::from_timestamp(token.expires_at, 0).unwrap();
    let expires_at = Some(expires_at);
    let expires_in = token.expires_at - chrono::Utc::now().timestamp();
    let expires_in = chrono::TimeDelta::new(expires_in, 0).unwrap();

    Ok(rspotify::Token {
        access_token: token.access_token,
        expires_in,
        expires_at,
        refresh_token: Some(token.refresh_token),
        scopes: rspotify::scopes!(token.scope),
    })
}

async fn create_listener(
    pool: &sqlx::PgPool,
    jam_id: &str,
    channel_name: &str,
) -> Result<PgListener, real_time::Error> {
    let mut listener = match PgListener::connect_with(pool).await {
        Ok(listener) => listener,
        Err(e) => {
            return Err(real_time::Error::Database(e.to_string()));
        }
    };

    let channel = format!("{}_{}", jam_id, channel_name);
    match listener.listen(&channel).await {
        Ok(_) => Ok(listener),
        Err(e) => Err(real_time::Error::Database(e.to_string())),
    }
}
