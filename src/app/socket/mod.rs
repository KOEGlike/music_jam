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
use read::read;
mod write;
use write::write;

#[derive(Debug, serde::Deserialize)]
pub struct QueryId {
    pub id: String,
}

#[derive(Debug, Clone)]
struct Id {
    pub id: String,
    pub jam_id: String,
}

#[derive(Clone, Debug)]
enum IdType {
    Host(Id),
    User(Id),
}

impl IdType {
    pub fn is_host(&self) -> bool {
        matches!(self, IdType::Host { .. })
    }
    pub fn is_user(&self) -> bool {
        matches!(self, IdType::User { .. })
    }
    pub fn id(&self) -> &str {
        match self {
            IdType::Host(id) => &id.id,
            IdType::User(id) => &id.id,
        }
    }
    pub fn jam_id(&self) -> &str {
        match self {
            IdType::Host(id) => &id.jam_id,
            IdType::User(id) => &id.jam_id,
        }
    }
}

pub async fn socket(
    ws: WebSocketUpgrade,
    Query(id): Query<QueryId>,
    State(state): State<AppState>,
) -> Response {
    leptos::logging::log!("ws: {:?}", id);
    ws.on_upgrade(|socket| handle_socket(socket, state, id.id))
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

async fn handle_error(error: real_time::Error, close: bool, sender: &mpsc::Sender<ws::Message>) {
    eprintln!("Error: {:?}", error);

    if close {
        let close_frame = error.to_close_frame();
        sender
            .send(ws::Message::Close(Some(close_frame)))
            .await
            .unwrap();
    } else {
        let update = real_time::Update::Error(error);
        let bin = rmp_serde::to_vec(&update).unwrap();
        sender.send(ws::Message::Binary(bin)).await.unwrap();
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
        return Ok(IdType::Host (
            Id {
                id: id.to_string(),
                jam_id,
            }
        ));
    }

    let user_check = sqlx::query!("SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)", id)
        .fetch_one(pool)
        .await?;

    if user_check.exists.unwrap_or(false) {
        let jam_id = sqlx::query!("SELECT jam_id FROM users WHERE id = $1", id)
            .fetch_one(pool)
            .await?
            .jam_id;
        return Ok(IdType::User (
            Id {
                id: id.to_string(),
                jam_id,
            }
        ));
    }

    Err(sqlx::Error::RowNotFound)
}

async fn send(
    mut receiver: mpsc::Receiver<ws::Message>,
    mut sender: SplitSink<WebSocket, ws::Message>,
) {
    while let Some(msg) = receiver.recv().await {
        let close_connection = matches!(msg, ws::Message::Close(_));

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
