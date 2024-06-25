use super::IdType;
use crate::general_types::*;
use axum::extract::ws::{self, WebSocket};
use futures_util::{stream::SplitStream, StreamExt};
use real_time::Update;
use sqlx::Postgres;
use tokio::sync::mpsc;

pub async fn read(
    mut receiver: SplitStream<WebSocket>,
    sender: mpsc::Sender<ws::Message>,
    id: IdType,
    app_state: AppState,
) {
    while let Some(message) = receiver.next().await {
        let message = match message {
            Ok(m) => m,
            Err(e) => {
                eprintln!("Error receiving message: {:?}", e);
                break;
            }
        };

        let message: real_time::Request =
            match rmp_serde::from_slice(message.into_data().as_slice()) {
                Ok(m) => m,
                Err(e) => {
                    use real_time::Error;
                    let error = Update::Error(Error::Decode(e.to_string()));
                    let bin = rmp_serde::to_vec(&error).unwrap();
                    if sender.send(ws::Message::Binary(bin)).await.is_err() {
                        eprintln!("Error sending message: {:?}", e);
                    }
                    return;
                }
            };

        match message {
            real_time::Request::RemoveUser { user_id } => {
                kick_user(user_id, host_id, &app_state.db.pool)
                    .await
                    .unwrap()
            }
            real_time::Request::AddSong { song_id } => todo!(),
            real_time::Request::RemoveSong { song_id } => todo!(),
        }
    }
}

async fn kick_user(
    user_id: String,
    host_id: String,
    jam_id: String,
    pool: &sqlx::Pool<Postgres>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "DELETE FROM users WHERE id=$1 AND jam_id=$2; ",
        user_id,
        jam_id
    )
    .execute(pool)
    .await?;

    sqlx::query!(
        "SELECT pg_notify( (SELECT id FROM jams WHERE host_id=$1) || 'users','')",
        host_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn add_song(
    song_id: String,
    user_id: String,
    jam_id: String,
    pool: &sqlx::Pool<Postgres>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO songs (id, user_id) VALUES ($1, $2);",
        song_id,
        user_id,
    )
        .execute(pool)
        .await?;

    sqlx::query!("SELECT pg_notify($1 || 'songs','')", jam_id)
        .execute(pool)
        .await?;
    Ok(())
}

async fn remove_song(
    song_id: String,
    jam_id: String,
    pool: &sqlx::Pool<Postgres>,
) -> Result<(), sqlx::Error> {
    sqlx::query!("DELETE FROM songs WHERE id=$1;", song_id)
        .execute(pool)
        .await?;

    sqlx::query!("SELECT pg_notify($1 || 'songs','')", jam_id)
        .execute(pool)
        .await?;
    Ok(())
}
