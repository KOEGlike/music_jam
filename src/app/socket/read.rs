use super::handle_error;
use crate::app::general::*;
use axum::extract::ws::{self, WebSocket};
use futures_util::{stream::SplitStream, StreamExt};
use tokio::sync::mpsc;

pub async fn read(
    mut receiver: SplitStream<WebSocket>,
    sender: mpsc::Sender<ws::Message>,
    id: IdType,
    app_state: AppState,
) {
    let pool = &app_state.db.pool;

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
            real_time::Request::KickUser { user_id } => {
                let host_id = match only_host(
                    &id,
                    "Only hosts can kick users, this is a bug, terminating socket connection",
                    &sender,
                )
                .await
                {
                    Ok(id) => &id.id,
                    Err(_) => break,
                };

                if let Err(error) = kick_user(&user_id, host_id, pool).await {
                    handle_error(error.into(), false, &sender).await;
                };
            }
            real_time::Request::AddSong { song_id } => {
                let id = match only_user(
                    &id,
                    "Only users can add songs, this is a bug, terminating socket connection",
                    &sender,
                )
                .await
                {
                    Ok(id) => id,
                    Err(_) => break,
                };

                if let Err(error) = add_song(&song_id, &id.id, &id.jam_id, pool).await {
                    handle_error(error, false, &sender).await;
                };
            }
            real_time::Request::RemoveSong { song_id } => {
                let jam_id = id.jam_id();
                if let Err(error) = remove_song(&song_id, jam_id, pool).await {
                    handle_error(error.into(), false, &sender).await;
                };
            }
            real_time::Request::AddVote { song_id } => {
                let id = match only_user(
                    &id,
                    "Only users can vote, this is a bug, terminating socket connection",
                    &sender,
                )
                .await
                {
                    Ok(id) => id,
                    Err(_) => break,
                };

                if let Err(error) = add_vote(&song_id, &id.id, &id.jam_id, pool).await {
                    handle_error(error.into(), false, &sender).await;
                };
            }
            real_time::Request::RemoveVote { song_id } => {
                let id = match only_host(
                    &id,
                    "Only users can remove votes, this is a bug, terminating socket connection",
                    &sender,
                )
                .await
                {
                    Ok(id) => id,
                    Err(_) => break,
                };

                if let Err(error) = remove_vote(&song_id, &id.id, &id.jam_id, pool).await {
                    handle_error(error.into(), false, &sender).await;
                };
            }
            real_time::Request::Update => {
                let err = tokio::join!(
                    notify(real_time::Channels::Songs, id.jam_id(), pool),
                    notify(real_time::Channels::Users, id.jam_id(), pool),
                    notify(real_time::Channels::Votes, id.jam_id(), pool)
                );

                if let Err(e) = err.0 {
                    handle_error(e.into(), false, &sender).await;
                } 
                if let Err(e) = err.1 {
                    handle_error(e.into(), false, &sender).await;
                }
                if let Err(e) = err.2 {
                    handle_error(e.into(), false, &sender).await;
                }
            }
            real_time::Request::Search { query } => {
                let songs = match search(&query, pool, id.jam_id()).await {
                    Ok(songs) => songs,
                    Err(e) => {
                        handle_error(e, false, &sender).await;
                        continue;
                    }
                };

                let update = real_time::Update::Search(songs);
                let message = rmp_serde::to_vec(&update).unwrap();
                let message = ws::Message::Binary(message);
                if let Err(e) = sender.send(message).await {
                    eprintln!("Error sending message: {:?}", e);
                    break;
                }
            
            }
        }
    }
}

use super::Id;

async fn only_host<'a>(
    id: &'a IdType,
    message: &str,
    sender: &mpsc::Sender<ws::Message>,
) -> Result<&'a Id, ()> {
    match id {
        IdType::Host(id) => Ok(id),
        IdType::User { .. } => {
            let error = real_time::Error::Forbidden(message.to_string());

            handle_error(error, true, sender).await;
            Err(())
        }
    }
}

async fn only_user<'a>(
    id: &'a IdType,
    message: &str,
    sender: &mpsc::Sender<ws::Message>,
) -> Result<&'a Id, ()> {
    match id {
        IdType::User(id) => Ok(id),
        IdType::Host { .. } => {
            let error = real_time::Error::Forbidden(message.to_string());

            handle_error(error, true, sender).await;
            Err(())
        }
    }
}
