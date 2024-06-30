use super::{handle_error, IdType};
use crate::app::general::*;
use axum::extract::ws;
use sqlx::postgres::PgListener;
use std::future::Future;
use tokio::sync::mpsc;

pub async fn write(sender: mpsc::Sender<ws::Message>, id: IdType, app_state: AppState) {
    let pool = app_state.db.pool;

    let listen_songs = tokio::spawn(listen_songs(
        pool.clone(),
        id.jam_id().into(),
        sender.clone(),
    ));

    let listen_users = tokio::spawn(listen_users(
        pool.clone(),
        id.jam_id().into(),
        sender.clone(),
    ));

    let listen_votes = tokio::spawn(listen_votes(
        pool.clone(),
        id.jam_id().into(),
        sender.clone(),
    ));

    tokio::select! {
        _ = listen_songs => {},
        _ = listen_users => {},
        _ = listen_votes => {},
    }
}

async fn listen_songs(
    pool: sqlx::PgPool,
    jam_id: String,
    sender: mpsc::Sender<ws::Message>,
) -> Result<(), real_time::Error> {
    listen(
        &pool,
        &jam_id,
        sender,
        real_time::Channels::Songs,
        get_songs,
    )
    .await
}

async fn listen_users(
    pool: sqlx::PgPool,
    jam_id: String,
    sender: mpsc::Sender<ws::Message>,
) -> Result<(), real_time::Error> {
    listen(
        &pool,
        &jam_id,
        sender,
        real_time::Channels::Users,
        get_users,
    )
    .await
}

async fn listen_votes(
    pool: sqlx::PgPool,
    jam_id: String,
    sender: mpsc::Sender<ws::Message>,
) -> Result<(), real_time::Error> {
    listen(
        &pool,
        &jam_id,
        sender,
        real_time::Channels::Votes,
        get_votes,
    )
    .await
}

async fn listen<'a, T, Fu>(
    pool: &'a sqlx::PgPool,
    jam_id: &'a str,
    sender: mpsc::Sender<ws::Message>,
    channel: real_time::Channels,
    f: fn(&'a sqlx::PgPool, &'a str) -> Fu,
) -> Result<(), real_time::Error>
where
    T: Into<real_time::Update>,
    Fu: Future<Output = T> + 'a,
{
    let mut listener = create_listener(pool, jam_id, channel).await?;

    while let Ok(m) = listener.try_recv().await {
        if m.is_none() {
            let error = real_time::Error::Database("pool disconnected reconnecting...".to_string());
            handle_error(error, false, &sender).await;
            continue;
        }

        let update: real_time::Update = f(pool, jam_id).await.into();
        let bin = rmp_serde::to_vec(&update).unwrap();
        let message = ws::Message::Binary(bin);
        sender.send(message).await.unwrap();
    }

    Ok(())
}

async fn create_listener(
    pool: &sqlx::PgPool,
    jam_id: &str,
    channel: real_time::Channels,
) -> Result<PgListener, real_time::Error> {
    let mut listener = match PgListener::connect_with(pool).await {
        Ok(listener) => listener,
        Err(e) => {
            return Err(real_time::Error::Database(e.to_string()));
        }
    };

    let channel: String = channel.into();
    let channel = format!("{}_{}", jam_id, channel);
    match listener.listen(&channel).await {
        Ok(_) => Ok(listener),
        Err(e) => Err(real_time::Error::Database(e.to_string())),
    }
}
