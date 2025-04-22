use super::{Id, handle_error};
use crate::model::*;
use axum::extract::ws;
use sqlx::postgres::PgListener;
use tokio::sync::mpsc;

pub async fn write(sender: mpsc::Sender<ws::Message>, id: Id, app_state: AppState) {
    let pool = app_state.db.pool;
    let mut listener = match create_listener(&pool, &id).await {
        Ok(listener) => listener,
        Err(e) => {
            handle_error(e, false, &sender).await;
            return;
        }
    };

    while let Ok(m) = listener.try_recv().await {
        match m {
            None => {
                let error =
                    Error::Database("pool disconnected on listener, reconnecting...".to_string());
                handle_error(error, false, &sender).await;
                continue;
            }
            Some(message) => {
                let update = message.payload();
                let update: real_time::ChannelUpdate = match serde_json::from_str(update) {
                    Ok(update) => update,
                    Err(e) => {
                        let error = Error::Decode(format!(
                            "Error decoding message sent in listen/notify: {:#?}",
                            e
                        ));
                        handle_error(error, true, &sender).await;
                        break;
                    }
                };

                let mut changed = update.changed;
                let errors = update.errors;
                if id.is_host() {
                    changed.position = false;
                    changed.current_song = false;
                }

                let mut transaction = match pool.begin().await {
                    Ok(transaction) => transaction,
                    Err(e) => {
                        let error = Error::Database(format!("Error starting transaction: {:?}", e));
                        handle_error(error, true, &sender).await;
                        continue;
                    }
                };
                let message = real_time::Update::from_changed(changed, &id, &mut transaction)
                    .await
                    .error_vec(errors);
                if let Err(e) = transaction.commit().await {
                    let error = Error::Database(format!("Error committing transaction: {:?}", e));
                    handle_error(error, true, &sender).await;
                    continue;
                }

                let bin = match rmp_serde::to_vec(&message) {
                    Ok(bin) => bin,
                    Err(e) => {
                        let error =
                            Error::Decode(format!("Error encoding message sent in ws: {:?}", e));
                        handle_error(error, true, &sender).await;
                        break;
                    }
                };

                match sender.send(ws::Message::Binary(bin)).await {
                    Ok(_) => (),
                    Err(e) => {
                        eprintln!("Error sending ws send message: {:?}", e);
                        break;
                    }
                }
            }
        }
    }
}

async fn create_listener(pool: &sqlx::PgPool, id: &Id) -> Result<PgListener, Error> {
    let mut listener = match PgListener::connect_with(pool).await {
        Ok(listener) => listener,
        Err(e) => {
            return Err(Error::Database(e.to_string()));
        }
    };

    match listener.listen(id.jam_id()).await {
        Ok(_) => Ok(listener),
        Err(e) => Err(Error::Database(e.to_string())),
    }
}
