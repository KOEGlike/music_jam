use crate::general_types::*;

use axum::{
    extract::{ws::*, Query, State},
    response::Response,
};

use futures_util::{
    sink::SinkExt,
    stream::{SplitSink, SplitStream, StreamExt},
};

#[derive(Debug, serde::Deserialize)]
pub struct Id {
    pub id: String,
}

pub async fn socket(
    ws: WebSocketUpgrade,
    Query(id): Query<Id>,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state, id.id))
}

#[derive(Clone, Debug)]
enum IdType {
    Host { id: String, jam_id: String },
    User { id: String, jam_id: String },
}

impl IdType {
    fn jam_id(&self) -> &str {
        match self {
            IdType::Host { jam_id, .. } => jam_id,
            IdType::User { jam_id, .. } => jam_id,
        }
    }
    fn id(&self) -> &str {
        match self {
            IdType::Host { id, .. } => id,
            IdType::User { id, .. } => id,
        }
    }
}


use sqlx::{query, Pool, Postgres};
use std::result::Result;

async fn check_id_type(id: &str, pool: &Pool<Postgres>) -> Result<IdType, sqlx::Error> {
    // Check if the ID exists in the hosts table
    let host_check = sqlx::query!("SELECT EXISTS(SELECT 1 FROM hosts WHERE id = $1)", id)
        .fetch_one(pool)
        .await?;

    if host_check.exists.unwrap_or(false) {
        let jam_id = query!("SELECT id FROM jams WHERE host_id = $1", id)
            .fetch_one(pool)
            .await?
            .id;
        return Ok(IdType::Host {
            id: id.to_string(),
            jam_id,
        });
    }

    let user_check = sqlx::query!("SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)", id)
        .fetch_one(pool)
        .await?;

    if user_check.exists.unwrap_or(false) {
        let jam_id = query!("SELECT jam_id FROM users WHERE id = $1", id)
            .fetch_one(pool)
            .await?
            .jam_id;
        return Ok(IdType::User {
            id: id.to_string(),
            jam_id,
        });
    }

    Err(sqlx::Error::RowNotFound)
}

async fn handle_socket(socket: WebSocket, app_state: AppState, id: String) {
    let (sender, receiver) = socket.split();

    let id = match check_id_type(&id, &app_state.db.pool).await {
        Ok(id) => id,
        Err(e) => {
            eprintln!("Error checking id type: {:?}", e);
            return;
        }
    };

    tokio::spawn(write(sender, id.clone(), app_state.clone()));
    tokio::spawn(read(receiver, id.clone(), app_state.clone()));
}

async fn write(mut sender: SplitSink<WebSocket, Message>, id: IdType, app_state: AppState) {
    let pool = app_state.db.pool;

    let listener = sqlx::postgres::PgListener::connect_with(&pool).await;
    let mut listener = match listener {
        Ok(listener) => listener,
        Err(e) => {
            eprintln!("Error connecting to listener: {:?}", e);
            return;
        }
    };

    if let Err(e) = listener.listen(format!("{}users",id.jam_id()).as_str()).await {
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

        let result = match sqlx::query_as! {User, "SELECT * FROM users WHERE jam_id=$1", id.jam_id()}
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
        match sender.send(Message::Binary(bin)).await {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Error sending message: {:?}", e);
                continue;
            }
        }
    }
}

async fn read(mut receiver: SplitStream<WebSocket>, id: IdType, app_state: AppState) {
    while let Some(message) = receiver.next().await {
        match message {
            Ok(message) => {
                println!("Received message: {:?}", message);
            }
            Err(e) => {
                eprintln!("Error receiving message: {:?}", e);
                break;
            }
        }
    }
}

#[server]
pub async fn kick_user(user_id: String, host_id: String) -> Result<(), ServerFnError> {
    let app_state = expect_context::<AppState>();
    let pool = app_state.db.pool;
    let notify_query = sqlx::query!(
        "SELECT pg_notify( (SELECT id FROM jams WHERE host_id=$1) || 'users','')",
        host_id
    );
    let delete_query = sqlx::query!(
        "DELETE FROM users WHERE id=$1 AND jam_id IN (SELECT id FROM jams WHERE host_id=$2); ",
        user_id,
        host_id
    );
    delete_query.execute(&pool).await?;
    notify_query.execute(&pool).await?;
    Ok(())
}
