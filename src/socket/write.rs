use super::{handle_error, IdType};
use crate::general::*;
use axum::extract::ws;
use leptos::logging::log;
use sqlx::postgres::PgListener;
use std::future::Future;
use tokio::sync::mpsc;

pub async fn write(sender: mpsc::Sender<ws::Message>, id: IdType, app_state: AppState) {
    let pool = app_state.db.pool;

    let listen_songs = tokio::spawn(listen_songs(pool.clone(), id.clone(), sender.clone()));

    let listen_users = tokio::spawn(listen_users(
        pool.clone(),
        id.jam_id().into(),
        sender.clone(),
    ));

    let listen_votes = tokio::spawn(listen_votes(pool.clone(), id.clone(), sender.clone()));
    let listen_ended = tokio::spawn(listen_ended(
        pool.clone(),
        id.jam_id().into(),
        sender.clone(),
    ));

    match id {
        IdType::Host(_) => {
            tokio::select! {
                _ = listen_songs => {},
                _ = listen_users => {},
                _ = listen_votes => {},
                _ = listen_ended => {},
            }
        }
        IdType::User(_) => {
            let listen_position = tokio::spawn(listen_position(
                pool.clone(),
                id.jam_id().into(),
                sender.clone(),
            ));
            tokio::select! {
                _ = listen_songs => {},
                _ = listen_users => {},
                _ = listen_votes => {},
                _ = listen_ended => {},
                _ = listen_position => {},
            }
        }
    }
}

async fn listen_ended(
    pool: sqlx::PgPool,
    jam_id: String,
    sender: mpsc::Sender<ws::Message>,
) -> Result<(), Error> {
    listen(
        &pool,
        &jam_id,
        sender,
        types::real_time::Channels::Ended,
        || async { types::real_time::Update::Ended },
    )
    .await
}

async fn listen_songs(
    pool: sqlx::PgPool,
    id: IdType,
    sender: mpsc::Sender<ws::Message>,
) -> Result<(), Error> {
    listen(
        &pool,
        id.jam_id(),
        sender,
        types::real_time::Channels::Songs,
        || {
            log!("updated songs");
            get_songs(&pool, &id)
        },
    )
    .await
}

async fn listen_users(
    pool: sqlx::PgPool,
    jam_id: String,
    sender: mpsc::Sender<ws::Message>,
) -> Result<(), Error> {
    listen(
        &pool,
        &jam_id,
        sender,
        types::real_time::Channels::Users,
        || get_users(&pool, &jam_id),
    )
    .await
}

async fn listen_votes(
    pool: sqlx::PgPool,
    id: IdType,
    sender: mpsc::Sender<ws::Message>,
) -> Result<(), Error> {
    listen(
        &pool,
        id.jam_id(),
        sender,
        types::real_time::Channels::Votes,
        || get_votes(&pool, &id),
    )
    .await
}

async fn listen_position(
    pool: sqlx::PgPool,
    jam_id: String,
    sender: mpsc::Sender<ws::Message>,
) -> Result<(), Error> {
    let mut listener = create_listener(&pool, &jam_id, real_time::Channels::Position).await?;

    while let Ok(message) = listener.try_recv().await {
        let message = match message {
            None => {
                let error = Error::Database("pool disconnected reconnecting...".to_string());
                handle_error(error, false, &sender).await;
                continue;
            }
            Some(message) => message,
        };

        let percentage: f32 = message.payload().parse().unwrap();

        let update: types::real_time::Update = real_time::Update::Position(percentage);
        let bin = rmp_serde::to_vec(&update).unwrap();
        let message = ws::Message::Binary(bin);
        if let Err(e) = sender.send(message).await {
            eprintln!("Error sending ws listen message: {:?}", e);
            break;
        }
    }

    Ok(())
}

async fn listen<'a, T, Fu, F>(
    pool: &'a sqlx::PgPool,
    jam_id: &'a str,
    sender: mpsc::Sender<ws::Message>,
    channel: types::real_time::Channels,
    f: F,
) -> Result<(), Error>
where
    T: Into<types::real_time::Update>,
    Fu: Future<Output = T> + 'a,
    F: Fn() -> Fu,
{
    let mut listener = create_listener(pool, jam_id, channel).await?;

    while let Ok(m) = listener.try_recv().await {
        if m.is_none() {
            let error = Error::Database("pool disconnected reconnecting...".to_string());
            handle_error(error, false, &sender).await;
            continue;
        }

        let update: types::real_time::Update = f().await.into();
        let bin = rmp_serde::to_vec(&update).unwrap();
        let message = ws::Message::Binary(bin);
        if let Err(e) = sender.send(message).await {
            eprintln!("Error sending ws listen message: {:?}", e);
            break;
        }
    }

    Ok(())
}

async fn create_listener(
    pool: &sqlx::PgPool,
    jam_id: &str,
    channel: types::real_time::Channels,
) -> Result<PgListener, Error> {
    let mut listener = match PgListener::connect_with(pool).await {
        Ok(listener) => listener,
        Err(e) => {
            return Err(Error::Database(e.to_string()));
        }
    };

    let channel: String = channel.into();
    let channel = format!("{}_{}", jam_id, channel);
    match listener.listen(&channel).await {
        Ok(_) => Ok(listener),
        Err(e) => Err(Error::Database(e.to_string())),
    }
}
