
use crate::general::*;
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
use tokio::sync::mpsc;

mod read;
mod write;

pub async fn socket(
    ws: WebSocketUpgrade,
    Query(id): Query<QueryId>,
    State(state): State<AppState>,
) -> Response {
    leptos::logging::log!("ws: {:?}", id);
    ws.on_upgrade(|socket| handle_socket(socket, state, id.id))
}

#[derive(Debug, serde::Deserialize)]
pub struct QueryId {
    pub id: String,
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
    let recv_task = tokio::spawn(read::read(
        receiver,
        mpsc_sender.clone(),
        id.clone(),
        app_state.clone(),
    ));

    let send_task = tokio::spawn(write::write(
        mpsc_sender.clone(),
        id.clone(),
        app_state.clone(),
    ));

    if let Err(e) = notify_all(id.jam_id(), &app_state.db.pool).await {
        handle_error(e.into(), false, &mpsc_sender).await;
    }

    bridge_task.await.unwrap();
    send_task.abort();
    recv_task.abort();
}

async fn handle_error(error: Error, close: bool, sender: &mpsc::Sender<ws::Message>) {
    eprintln!("Error: {:?}", error);

    if close {
        let close_frame = error.to_close_frame();
        sender
            .send(ws::Message::Close(Some(close_frame)))
            .await
            .unwrap();
    } else {
        let update = types::real_time::Update::Error(error);
        let bin = rmp_serde::to_vec(&update).unwrap();
        sender.send(ws::Message::Binary(bin)).await.unwrap();
    }
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
                eprintln!("Error sending ws send message: {:?}", e);
                break;
            }
        }

        if close_connection {
            break;
        }
    }
    if let Err(e) = sender.close().await {
        eprintln!("Error closing ws connection: {:?}", e);
    };
}


