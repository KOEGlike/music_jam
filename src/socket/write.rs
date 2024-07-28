use super::{handle_error, IdType};
use crate::general::*;
use axum::extract::ws;
use leptos::logging::log;
use sqlx::{pool, postgres::PgListener};
use std::future::Future;
use std::sync::Arc;
use strum::IntoEnumIterator;
use tokio::sync::mpsc;
use tokio::task::JoinSet;

pub async fn write(sender: mpsc::Sender<ws::Message>, id: IdType, app_state: AppState) {
    let pool = app_state.db.pool;
    let id = Arc::new(id);
    let mut futures = JoinSet::new();

    for channel in real_time::Channels::iter() {
        let id = Arc::clone(&id);
        let sender = sender.clone();

        match channel {
            real_time::Channels::Songs => {
                let f = |_| {
                    let pool = pool.clone();
                    let id = Arc::clone(&id);
                    async move { get_songs(&pool, &id).await }
                };

                futures.spawn(listen(pool.clone(), id.jam_id(), sender, channel, f));
            }
            real_time::Channels::Votes => {
                let f = |_| {
                    let pool = pool.clone();
                    let id = Arc::clone(&id);
                    async move { get_votes(&pool, &id).await }
                };
                futures.spawn(listen(pool.clone(), id.jam_id(), sender, channel, f));
            }
            real_time::Channels::Users => {
                let f =  |_| {
                    let pool = pool.clone();
                    let id = Arc::clone(&id);
                    async move { get_users(&pool, id.jam_id()).await }
                };
                futures.spawn(listen(pool.clone(), id.jam_id(), sender, channel, f));
            }
            real_time::Channels::Ended => {
                let f = |_| async { real_time::Update::Ended };
                futures.spawn(listen(pool.clone(), id.jam_id(), sender, channel, f));
            }
            real_time::Channels::Position { .. } => {
                if id.is_host() {
                    continue;
                }
                let f = |m: sqlx::postgres::PgNotification| async move {
                    real_time::Update::Position {
                        percentage: m.payload().parse().unwrap(),
                    }
                };
                //futures.spawn(listen(&pool, id.jam_id(), sender, channel, f));
            }
            real_time::Channels::CurrentSong => {
                if id.is_host() {
                    continue;
                }
                let pool = pool.clone();
                let id = Arc::clone(&id);
                let f = move |_| {
                    let pool = pool.clone();
                    let id = Arc::clone(&id);
                    async move {
                        let song = match get_current_song(id.jam_id(), &pool).await {
                            Ok(song) => song,
                            Err(e) => {
                                return real_time::Update::Error(Error::Database(format!(
                                    "error getting current song: {:?}",
                                    e
                                )));
                            }
                        };
                        real_time::Update::CurrentSong(song)
                    }
                };
                futures.spawn(listen(pool.clone(), id.jam_id(), sender, channel, f));
            }
        };
    }
}

async fn listen<'a, T, Fu, F>(
    pool: sqlx::PgPool,
    jam_id: &'a str,
    sender: mpsc::Sender<ws::Message>,
    channel: types::real_time::Channels,
    f: F,
) -> Result<(), Error>
where
    T: Into<types::real_time::Update>,
    Fu: Future<Output = T> + 'a,
    F: Fn(sqlx::postgres::PgNotification) -> Fu,
{
    let mut listener = create_listener(&pool, jam_id, channel).await?;

    while let Ok(m) = listener.try_recv().await {
        match m {
            None => {
                let error = Error::Database("pool disconnected reconnecting...".to_string());
                handle_error(error, false, &sender).await;
                continue;
            }
            Some(message) => {
                let update: types::real_time::Update = f(message).await.into();
                let bin = rmp_serde::to_vec(&update).unwrap();
                let message = ws::Message::Binary(bin);
                if let Err(e) = sender.send(message).await {
                    eprintln!("Error sending ws listen message: {:?}", e);
                    break;
                }
            }
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
