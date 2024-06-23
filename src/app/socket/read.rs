use futures_util::{stream::SplitStream,StreamExt};
use sqlx::Postgres;
use crate::general_types::*;
use axum::extract::ws::{self, WebSocket};
use tokio::sync::mpsc;
use super::IdType;

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

        let message: real_time::RealTimeRequest =
            match rmp_serde::from_slice(message.into_data().as_slice()) {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("Error deserializing message: {:?}", e);
                    continue;
                }
            };

        match message {
            real_time::RealTimeRequest::AddUser { user } => todo!(),
            real_time::RealTimeRequest::RemoveUser { user_id, host_id } => todo!(),
            real_time::RealTimeRequest::AddSong { song_id, user_id } => todo!(),
            real_time::RealTimeRequest::RemoveSong { song_id, id } => todo!(),
        }
    }
}

async fn kick_user(
    user_id: String,
    host_id: String,
    pool: sqlx::Pool<Postgres>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "SELECT pg_notify( (SELECT id FROM jams WHERE host_id=$1) || 'users','')",
        host_id
    )
    .execute(&pool)
    .await?;

    sqlx::query!(
        "DELETE FROM users WHERE id=$1 AND jam_id IN (SELECT id FROM jams WHERE host_id=$2); ",
        user_id,
        host_id
    )
    .execute(&pool)
    .await?;
    Ok(())
}
