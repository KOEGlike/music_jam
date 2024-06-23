use axum::extract::ws;
use tokio::sync::mpsc;
use crate::general_types::*;
use super::IdType;

pub async fn write(sender: mpsc::Sender<ws::Message>, id: IdType, app_state: AppState) {
    let pool = app_state.db.pool;

    let listener = sqlx::postgres::PgListener::connect_with(&pool).await;
    let mut listener = match listener {
        Ok(listener) => listener,
        Err(e) => {
            eprintln!("Error connecting to listener: {:?}", e);
            return;
        }
    };

    if let Err(e) = listener
        .listen(format!("{}users", id.jam_id()).as_str())
        .await
    {
        eprintln!("Error listening to channel: {:?}", e);
        return;
    }

    loop {
        let notification = match listener.try_recv().await {
            Ok(Some(notification)) => notification,
            Err(e) => {
                eprintln!("Error receiving notification: {:?}", e);
                continue;
            }
            Ok(None) => {
                eprint!("disconnected from pool, reconnecting on next iteration");
                continue;
            }
        };

        println!("Received notification: {:#?}", notification);

        let result = match sqlx::query_as!(User, "SELECT * FROM users WHERE jam_id=$1", id.jam_id())
            .fetch_all(&pool)
            .await
        {
            Ok(result) => result,
            Err(e) => {
                eprintln!("Error fetching users: {:?}", e);
                continue;
            }
        };

        let bin = match rmp_serde::to_vec(&result) {
            Ok(bin) => bin,
            Err(e) => {
                eprintln!("Error serializing users: {:?}", e);
                continue;
            }
        };
        match sender.send(ws::Message::Binary(bin)).await {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Error sending message: {:?}", e);
                continue;
            }
        }
    }
}
