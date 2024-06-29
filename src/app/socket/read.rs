use super::{IdType, handle_error};
use crate::general_types::*;
use axum::extract::ws::{self, WebSocket};
use futures_util::{stream::SplitStream, StreamExt,};
use sqlx::Postgres;
use tokio::sync::mpsc;
use crate::app::general_functions::*;

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
                    let error = Error::Decode(e.to_string());
                    handle_error(error, true, &sender).await;
                    break;
                }
            };

        match message {
            real_time::Request::RemoveUser { user_id } => {
                let host_id = match &id {
                    IdType::Host { id, .. } => id,
                    IdType::User { .. } => {
                        let error = real_time::Error::Forbidden(
                            "Only the host can kick users, if you see this in prod this is a bug"
                                .to_string(),
                        );

                        handle_error(error, true, &sender).await;
                        break;
                    }
                };

                if let Err(error) = kick_user(&user_id, host_id, &app_state.db.pool).await {
                    handle_error(error.into(), false, &sender).await;
                };
            }
            real_time::Request::AddSong { song_id } => {
                let (user_id, jam_id) = match &id {
                    IdType::User { id, jam_id } => (id, jam_id),
                    IdType::Host { .. } => {
                        let error = real_time::Error::Forbidden(
                            "Only users can add songs, if you see this in prod this is a bug"
                                .to_string(),
                        );

                        handle_error(error, true, &sender).await;
                        break;
                    }
                };

                if let Err(error) = add_song(&song_id, user_id, jam_id, &app_state.db.pool).await {
                    handle_error(error.into(), false, &sender).await;
                };
            }
            real_time::Request::RemoveSong { song_id } => {
                let jam_id = match &id {
                    IdType::Host { jam_id, .. } => jam_id,
                    IdType::User { .. } => {
                        let error = real_time::Error::Forbidden(
                            "Only the host can remove songs, if you see this in prod this is a bug"
                                .to_string(),
                        );

                        handle_error(error, true, &sender).await;
                        break;
                    }
                };

                if let Err(error) = remove_song(&song_id, jam_id, &app_state.db.pool).await {
                    handle_error(error.into(), false, &sender).await;
                };
            }
            real_time::Request::AddVote { song_id } => {
                let (user_id, jam_id) = match &id {
                    IdType::User { id, jam_id } => (id, jam_id),
                    IdType::Host { .. } => {
                        let error = real_time::Error::Forbidden(
                            "Only users can vote, if you see this in prod this is a bug"
                                .to_string(),
                        );

                        handle_error(error, true, &sender).await;
                        break;
                    }
                };

                if let Err(error) = add_vote(&song_id, user_id, jam_id, &app_state.db.pool).await {
                    handle_error(error.into(), false, &sender).await;
                };
            }
            real_time::Request::RemoveVote { song_id } => {
                let (user_id, jam_id) = match &id {
                    IdType::User { id, jam_id } => (id, jam_id),
                    IdType::Host { .. } => {
                        let error = real_time::Error::Forbidden(
                            "Only users can vote, if you see this in prod this is a bug"
                                .to_string(),
                        );
                        handle_error(error, true, &sender).await;
                        break;
                    }
                };

                if let Err(error) = remove_vote(&song_id, user_id, jam_id, &app_state.db.pool).await
                {
                    handle_error(error.into(), false, &sender).await;
                };
            }
        }
    }
}






