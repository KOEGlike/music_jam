use super::handle_error;
use crate::general::*;
use axum::extract::ws::{self, WebSocket};
use futures_util::{stream::SplitStream, StreamExt};
use tokio::sync::mpsc;

pub async fn read(
    mut receiver: SplitStream<WebSocket>,
    sender: mpsc::Sender<ws::Message>,
    id: IdType,
    app_state: AppState,
) {
    let pool = &app_state.db.pool.clone();

    let credentials = app_state.spotify_credentials;

    while let Some(message) = receiver.next().await {
        let message = match message {
            Ok(m) => m,
            Err(e) => {
                eprintln!("Error receiving ws message: {:?}", e);
                break;
            }
        };

        let message: types::real_time::Request =
            match rmp_serde::from_slice(message.into_data().as_slice()) {
                Ok(m) => m,
                Err(e) => {
                    use Error;
                    let error =
                        Error::Decode(format!("Error decoding message sent in ws: {:?}", e));
                    handle_error(error, true, &sender).await;
                    break;
                }
            };

        match message {
            real_time::Request::KickUser { user_id } => {
                if let Err(error) = kick_user(&user_id, pool).await {
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

                if let Err(error) =
                    add_song(&song_id, &id.id, &id.jam_id, pool, credentials.clone()).await
                {
                    handle_error(error, false, &sender).await;
                };
            }
            real_time::Request::RemoveSong { song_id } => {
                if let Err(error) = remove_song(&song_id, &id, pool).await {
                    handle_error(error, false, &sender).await;
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

                if let Err(error) = add_vote(&song_id, id, pool).await {
                    handle_error(error, false, &sender).await;
                };
            }
            real_time::Request::RemoveVote { song_id } => {
                let id = match only_user(
                    &id,
                    "Only users can remove votes, this is a bug, terminating socket connection",
                    &sender,
                )
                .await
                {
                    Ok(id) => id,
                    Err(_) => break,
                };

                if let Err(error) = remove_vote(&song_id, id, pool).await {
                    handle_error(error, false, &sender).await;
                };
            }
            real_time::Request::Update => {
                if let Err(e) = notify_all(id.jam_id(), pool).await {
                    handle_error(e, false, &sender).await;
                }
            }
            real_time::Request::Search { query } => {
                let id = match only_user(
                    &id,
                    "Only users can search, this is a bug, terminating socket connection",
                    &sender,
                )
                .await
                {
                    Ok(id) => id,
                    Err(_) => break,
                };

                let songs = match search(&query, pool, &id.jam_id, credentials.clone()).await {
                    Ok(songs) => songs,
                    Err(e) => {
                        handle_error(e, false, &sender).await;
                        continue;
                    }
                };

                let update = types::real_time::Update::Search(songs);
                let message = rmp_serde::to_vec(&update).unwrap();
                let message = ws::Message::Binary(message);
                if let Err(e) = sender.send(message).await {
                    eprintln!("Error sending ws message: {:?}", e);
                    break;
                }
            }
            real_time::Request::ResetVotes => {
                let id = match only_host(
                    &id,
                    "Only hosts can reset votes, this is a bug, terminating socket connection",
                    &sender,
                )
                .await
                {
                    Ok(id) => id,
                    Err(_) => break,
                };

                if let Err(error) = reset_votes(&id.jam_id, pool).await {
                    let error = Error::Database(format!("Error resetting votes: {}", error));
                    handle_error(error, false, &sender).await;
                };
            }
            real_time::Request::Position { percentage } => {
                let id = match only_host(
                    &id,
                    "Only a host can update the current position of a song",
                    &sender,
                )
                .await
                {
                    Ok(id) => id,
                    Err(_) => break,
                };

                if let Err(e)=set_current_song_position(&id.jam_id, percentage, pool).await {
                    handle_error(e, false, &sender).await;
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
            let error = Error::Forbidden(message.to_string());

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
            let error = Error::Forbidden(message.to_string());

            handle_error(error, true, sender).await;
            Err(())
        }
    }
}
