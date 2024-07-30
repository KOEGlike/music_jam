use super::{handle_error, IdType};
use crate::general::*;
use axum::extract::ws;
use sqlx::postgres::PgListener;
use tokio::sync::mpsc;

pub async fn write(sender: mpsc::Sender<ws::Message>, id: IdType, app_state: AppState) {
    let pool = app_state.db.pool;
    let mut listener = match create_listener(&pool, &id).await{
        Ok(listener)=>listener,
        Err(e)=>{
            handle_error(e, false, &sender).await;
            return;
        }
    };

    while let Ok(m) = listener.try_recv().await {
        match m {
            None => {
                let error = Error::Database("pool disconnected reconnecting...".to_string());
                handle_error(error, false, &sender).await;
                continue;
            }
            Some(message) => {
                let update=message.payload();
                let mut update:real_time::Update = match serde_json::from_str(&update) {
                    Ok(update) => update,
                    Err(e) => {
                        let error = Error::Decode(format!("Error decoding message sent in ws: {:?}", e));
                        handle_error(error, true, &sender).await;
                        break;
                    }
                };

                if update.songs.is_some() {
                    update=update.songs_from_jam(&id, &pool).await;
                }
                if update.votes.is_some() {
                    update=update.users_from_jam(&id, &pool).await;
                }

                let bin = match rmp_serde::to_vec(&update) {
                    Ok(bin) => bin,
                    Err(e) => {
                        let error = Error::Decode(format!("Error encoding message sent in ws: {:?}", e));
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

async fn create_listener(
    pool: &sqlx::PgPool,
    id:&IdType,
) -> Result<PgListener, Error> {
    let mut listener = match PgListener::connect_with(pool).await {
        Ok(listener) => listener,
        Err(e) => {
            return Err(Error::Database(e.to_string()));
        }
    };

    let channel: String = match id{
        IdType::Host(_)=>"host".to_string(),
        IdType::User(_)=>"user".to_string(),
    };
    let channel = format!("{}_{}", id.jam_id(), channel);
    match listener.listen(&channel).await {
        Ok(_) => Ok(listener),
        Err(e) => Err(Error::Database(e.to_string())),
    }
}

