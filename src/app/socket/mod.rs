use crate::general_types::*;
use axum::{
    extract::{
        ws::{self, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    response::Response,
};
use futures_util::{
    sink::SinkExt,
    stream::{SplitSink, StreamExt},
};
use sqlx::Postgres;
use std::result::Result;
use tokio::sync::mpsc;

mod read;
mod write;

use read::read;
use write::write;

#[derive(Debug, serde::Deserialize)]
pub struct Id {
    pub id: String,
}

#[derive(Clone, Debug)]
enum IdType {
    Host { id: String, jam_id: String },
    User { id: String, jam_id: String },
}

pub async fn socket(
    ws: WebSocketUpgrade,
    Query(id): Query<Id>,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state, id.id))
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

async fn check_id_type(id: &str, pool: &sqlx::Pool<Postgres>) -> Result<IdType, sqlx::Error> {
    // Check if the ID exists in the hosts table
    let host_check = sqlx::query!("SELECT EXISTS(SELECT 1 FROM hosts WHERE id = $1)", id)
        .fetch_one(pool)
        .await?;

    if host_check.exists.unwrap_or(false) {
        let jam_id = sqlx::query!("SELECT id FROM jams WHERE host_id = $1", id)
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
        let jam_id = sqlx::query!("SELECT jam_id FROM users WHERE id = $1", id)
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

async fn send(
    mut receiver: mpsc::Receiver<ws::Message>,
    mut sender: SplitSink<WebSocket, ws::Message>,
) {
    
    while let Some(msg) = receiver.recv().await {
        let close_connection = if let ws::Message::Close(_) = msg {
            true
        } else {
            false
        };

        match sender.send(msg).await {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Error sending message: {:?}", e);
                break;
            }
        }

        if close_connection {
            break;
        }
    }
}

async fn handle_socket(socket: WebSocket, app_state: AppState, id: String) {
    let (sender, receiver) = socket.split();
    let (mpsc_sender, mpsc_receiver) = mpsc::channel(3);

    let id = match check_id_type(&id, &app_state.db.pool).await {
        Ok(id) => id,
        Err(e) => {
            eprintln!("Error checking id type: {:?}", e);
            return;
        }
    };

    let bridge_task = tokio::spawn(send(mpsc_receiver, sender));
    let send_task = tokio::spawn(write(mpsc_sender.clone(), id.clone(), app_state.clone()));
    let recv_task = tokio::spawn(read(
        receiver,
        mpsc_sender.clone(),
        id.clone(),
        app_state.clone(),
    ));

    bridge_task.await.unwrap();
    send_task.abort();
    recv_task.abort();
}
