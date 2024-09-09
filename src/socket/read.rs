use super::handle_error;
use crate::model::*;
use axum::extract::ws::{self, WebSocket};
use futures_util::{stream::SplitStream, StreamExt};
use real_time::SearchResult;
use tokio::sync::mpsc;

pub async fn read(
    mut receiver: SplitStream<WebSocket>,
    sender: mpsc::Sender<ws::Message>,
    id: Id,
    app_state: AppState,
) {
    let pool = &app_state.db.pool.clone();
    let credentials = app_state.spotify_credentials;

    while let Some(message) = receiver.next().await {
        let message = match message {
            Ok(m) => m,
            Err(e) => {
                eprintln!("Error receiving ws message: {:#?}", e);
                break;
            }
        };

        tokio::spawn(handle_message(
            message,
            sender.clone(),
            id.clone(),
            pool.clone(),
            credentials.clone(),
        ));
    }
}

async fn handle_message(
    message: ws::Message,
    sender: mpsc::Sender<ws::Message>,
    id: Id,
    pool: sqlx::PgPool,
    credentials: SpotifyCredentials,
) {
    let pool = &pool;
    let message: types::real_time::Request = match rmp_serde::from_slice(&message.into_data()) {
        Ok(m) => m,
        Err(e) => {
            use Error;
            let error = Error::Decode(format!("Error decoding message sent in ws: {:#?}", e));
            handle_error(error, true, &sender).await;
            return;
        }
    };
    let mut changed = real_time::Changed::new();
    let mut errors: Vec<Error> = Vec::new();

    match message {
        real_time::Request::KickUser { user_id } => {
            let your_id = match &id.id {
                IdType::User(id) => id,
                IdType::Host(_) | IdType::General => {
                    let error = Error::Forbidden(
                        "Only users can kick other users, this is a bug, terminating socket connection"
                            .to_string(),
                    );
                    handle_error(error, true, &sender).await;
                    return;
                }
            };
            if !(&user_id == your_id || user_id.is_empty()) {
                let error = Error::Forbidden(
                    "A user only can kick themselves, this is a bug, terminating socket connection"
                        .to_string(),
                );
                handle_error(error, true, &sender).await;
                return;
            }
            let mut user_id = &user_id;
            if user_id.is_empty() {
                if id.is_user() {
                    user_id = your_id;
                } else {
                    errors.push(Error::InvalidRequest("No user id provided".to_string()));
                }
            }
            if errors.is_empty() {
                match kick_user(user_id, pool).await {
                    Ok(changed_new) => {
                        changed = changed.merge_with_other(changed_new);
                    }
                    Err(e) => {
                        errors.push(e.into());
                    }
                };
            }
        }
        real_time::Request::AddSong { song_id } => {
            let your_id = match only_user(
                &id,
                "Only users can add songs, this is a bug, terminating socket connection",
                &sender,
            )
            .await
            {
                Ok(id) => id,
                Err(_) => return,
            };

            match add_song(&song_id, your_id, id.jam_id(), pool, credentials.clone()).await {
                Ok(changed_new) => {
                    changed = changed.merge_with_other(changed_new);
                }
                Err(e) => {
                    errors.push(e);
                }
            };
        }
        real_time::Request::RemoveSong { song_id } => {
            match remove_song(&song_id, &id, pool).await {
                Ok(changed_new) => {
                    changed = changed.merge_with_other(changed_new);
                }
                Err(e) => {
                    errors.push(e);
                }
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
                Err(_) => return,
            };

            match add_vote(&song_id, id, pool).await {
                Ok(changed_new) => {
                    changed = changed.merge_with_other(changed_new);
                }
                Err(e) => {
                    errors.push(e);
                }
            };
        }
        real_time::Request::RemoveVote { song_id } => {
            let your_id = match only_user(
                &id,
                "Only users can remove votes, this is a bug, terminating socket connection",
                &sender,
            )
            .await
            {
                Ok(id) => id,
                Err(_) => return,
            };

            match remove_vote(&song_id, your_id, pool).await {
                Ok(changed_new) => {
                    changed = changed.merge_with_other(changed_new);
                }
                Err(e) => {
                    errors.push(e);
                }
            };
        }
        real_time::Request::Search {
            query,
            id: search_id,
        } => {
            if only_user(
                &id,
                "Only users can search, this is a bug, terminating socket connection",
                &sender,
            )
            .await
            .is_err()
            {
                return;
            }

            let songs = match search(&query, pool, id.jam_id(), credentials.clone()).await {
                Ok(songs) => songs,
                Err(e) => {
                    handle_error(e, false, &sender).await;
                    return;
                }
            };

            let update = real_time::Update::new().search(SearchResult { songs, search_id });
            let message = rmp_serde::to_vec(&update).unwrap();
            let message = ws::Message::Binary(message);
            if let Err(e) = sender.send(message).await {
                eprintln!("Error sending ws message: {:?}", e);
                return;
            }
        }
        real_time::Request::Position { percentage } => {
            if only_host(
                &id,
                "Only a host can update the current position of a song, this is a bug, terminating socket connection",
                &sender,
            )
            .await
            .is_err() {
                return;
            }

            match set_current_song_position(id.jam_id(), percentage, pool).await {
                Ok(changed_new) => {
                    changed = changed.merge_with_other(changed_new);
                }
                Err(e) => {
                    errors.push(e);
                }
            };
        }
        real_time::Request::NextSong => {
            if  only_host(
                &id,
        "Only a host can set the current song, this is a bug, terminating socket connection", 
                &sender
            )
                .await.is_err(){return;}

            match next_song(id.jam_id.clone(), pool, credentials.clone()).await {
                Ok(changed_new) => {
                    changed = changed.merge_with_other(changed_new);
                }
                Err(e) => {
                    errors.push(e);
                }
            }
        }
    }


    if let Err(e) = notify(changed, errors, id.jam_id(), pool).await {
        handle_error(e.into(), false, &sender).await;
    }
}

use super::Id;

///returns id of host, if the id is not a host, it returns an error
async fn only_host<'a>(
    id: &'a Id,
    message: &str,
    sender: &mpsc::Sender<ws::Message>,
) -> Result<&'a String, ()> {
    match &id.id {
        IdType::Host(id) => Ok(id),
        _ => {
            let error = Error::Forbidden(message.to_string());

            handle_error(error, true, sender).await;
            Err(())
        }
    }
}

/// returns the user id if the id is a user, otherwise sends an error message and returns an error
async fn only_user<'a>(
    id: &'a Id,
    message: &str,
    sender: &mpsc::Sender<ws::Message>,
) -> Result<&'a String, ()> {
    match &id.id {
        IdType::User(id) => Ok(id),
        _ => {
            let error = Error::Forbidden(message.to_string());

            handle_error(error, true, sender).await;
            Err(())
        }
    }
}
