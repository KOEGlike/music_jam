use super::{handle_error, IdType};
use crate::app::general_functions::*;
use crate::general_types::*;
use axum::extract::ws;
use sqlx::postgres::PgListener;
use std::future::Future;
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
    listen(&pool, &jam_id, sender, "songs", get_songs).await
}



async fn listen<T,Fu>(
    pool: &sqlx::PgPool,
    jam_id: &str,
    sender: mpsc::Sender<ws::Message>,
    channel_name: &str,
    f: fn(&sqlx::PgPool, &str) -> Fu,
) -> Result<(), real_time::Error>
where
    T: Into<real_time::Update>,
    Fu: Future<Output = T>+'static,
{
    let mut listener = create_listener(&pool, &jam_id, channel_name).await?;

    while let Ok(m) = listener.try_recv().await {
        if m.is_none() {
            let error = real_time::Error::Database("pool disconnected reconnecting...".to_string());
            handle_error(error, false, &sender).await;
            continue;
        }

        let update:real_time::Update = f(&pool, &jam_id).await.into();
        let bin = rmp_serde::to_vec(&update).unwrap();
        let message = ws::Message::Binary(bin);
        sender.send(message).await.unwrap();
    }

    Ok(())
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
