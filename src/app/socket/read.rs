use super::{IdType, handle_error};
use crate::general_types::*;
use axum::extract::ws::{self, WebSocket};
use futures_util::{stream::SplitStream, StreamExt,};
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
                    handle_error(error, false, &sender).await;
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
                    handle_error(error, false, &sender).await;
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
                    handle_error(error, false, &sender).await;
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
                    handle_error(error, false, &sender).await;
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
                    handle_error(error, false, &sender).await;
                };
            }
        }
    }
}



async fn notify(
    chanel: &str,
    jam_id: &str,
    pool: &sqlx::Pool<Postgres>,
) -> Result<(), real_time::Error> {
    let chanel=format!("{}_{}",jam_id,chanel);
    let res = sqlx::query!("SELECT pg_notify($1,'notified')",chanel)
        .execute(pool)
        .await;
    if let Err(e) = res {
        return Err(real_time::Error::Database(e.to_string()));
    }
    Ok(())
}

async fn kick_user(
    user_id: &str,
    host_id: &str,
    pool: &sqlx::Pool<Postgres>,
) -> Result<(), real_time::Error> {
    //check if the jam that the user is in is owned by the host
    struct JamId{
        id: String
    }
    
    let res = sqlx::query_as!(JamId,"SELECT id FROM jams WHERE host_id=$1;", host_id)
        .fetch_one(pool)
        .await;

    let jam_id=match res{
        Ok(id) => id.id,
        Err(e) => {
            return Err(real_time::Error::Database(e.to_string()));
        }
    };
    
    let res = sqlx::query!(
        "DELETE FROM users WHERE id=$1 AND jam_id=$2; ",
        user_id,
        jam_id
    )
    .execute(pool)
    .await;
    if let Err(e) = res {
        return Err(real_time::Error::Database(e.to_string()));
    }

    notify("users", &jam_id, pool).await?;
    Ok(())
}

async fn add_song(
    song_id: &str,
    user_id: &str,
    jam_id: &str,
    pool: &sqlx::Pool<Postgres>,
) -> Result<(), real_time::Error> {
    let res = sqlx::query!(
        "INSERT INTO songs (id, user_id) VALUES ($1, $2)",
        song_id,
        user_id,
    )
    .execute(pool)
    .await;

    if let Err(e) = res {
        return Err(real_time::Error::Database(e.to_string()));
    }

    notify("songs", jam_id, pool).await?;
    Ok(())
}

async fn remove_song(
    song_id: &str,
    jam_id: &str,
    pool: &sqlx::Pool<Postgres>,
) -> Result<(), real_time::Error> {
    let res = sqlx::query!("DELETE FROM songs WHERE id=$1;", song_id)
        .execute(pool)
        .await;

    if let Err(e) = res {
        return Err(real_time::Error::Database(e.to_string()));
    }

    notify("songs", jam_id, pool).await?;
    Ok(())
}

async fn add_vote(
    song_id: &str,
    user_id: &str,
    jam_id: &str,
    pool: &sqlx::Pool<Postgres>,
) -> Result<(), real_time::Error> {
    let res = sqlx::query!(
        "INSERT INTO votes (song_id, user_id) VALUES ($1, $2);",
        song_id,
        user_id,
    )
    .execute(pool)
    .await;

    if let Err(e) = res {
        return Err(real_time::Error::Database(e.to_string()));
    }

    notify("votes", jam_id, pool).await?;
    Ok(())
}

async fn remove_vote(
    song_id: &str,
    user_id: &str,
    jam_id: &str,
    pool: &sqlx::Pool<Postgres>,
) -> Result<(), real_time::Error> {
    let res = sqlx::query!(
        "DELETE FROM votes WHERE song_id=$1 AND user_id=$2;",
        song_id,
        user_id,
    )
    .execute(pool)
    .await;

    if let Err(e) = res {
        return Err(real_time::Error::Database(e.to_string()));
    }

    notify("votes", jam_id, pool).await?;
    Ok(())
}
