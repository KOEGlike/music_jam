use crate::model::*;
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
    println!("ws: {:?}", id);
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

    let checkup = if id.is_host() {
        let jam_id = id.jam_id;
        let pool = app_state.db.pool.clone();
        let spotify_credentials = app_state.spotify_credentials.clone();
        let handle = tokio::spawn(occasional_notify(
            pool.clone(),
            jam_id.clone(),
            spotify_credentials,
        ));
        Some(handle)
    } else {
        None
    };

    if let Err(e) = bridge_task.await {
        eprintln!("Error in bridge task: {:?}", e);
    };
    send_task.abort();
    recv_task.abort();
    if let Some(handle) = checkup {
        handle.abort();
    }
}

async fn occasional_notify(
    pool: sqlx::PgPool,
    jam_id: String,
    spotify_credentials: SpotifyCredentials,
) -> Result<(), Error> {
    use std::time::Duration;
    while dose_jam_exist(&jam_id, &pool).await.unwrap_or(true) {
        println!("Occasional notify");
        if let Err(e) = notify(real_time::Changed::all(), vec![], &jam_id, &pool).await {
            eprintln!("Error notifying all, in occasional notify: {:?}", e);
        };
        tokio::spawn(play_the_current_song_if_player_is_not_playing_it(
            jam_id.clone(),
            pool.clone(),
            spotify_credentials.clone(),
        ));
        tokio::time::sleep(Duration::from_secs(10)).await;
    }

    Ok(())
}

async fn play_the_current_song_if_player_is_not_playing_it(
    jam_id: String,
    pool: sqlx::PgPool,
    credentials: SpotifyCredentials,
) -> Result<(), Error> {
    use crate::model::functions::{get_current_song_from_player, play_song};
    let jam_id = &jam_id;
    let pool = &pool;
    let current_song = get_current_song(jam_id, pool).await?;
    let song = match current_song {
        Some(song) => song,
        None => return Ok(()),
    };
    let player_current_song =
        get_current_song_from_player(jam_id, pool, credentials.clone()).await?;
    if player_current_song
        .as_ref()
        .map(|s| s.spotify_id != song.spotify_id)
        .unwrap_or(true)
    {
        println!(
            "playing song: {:?}, set away from: {}",
            song.name,
            player_current_song.map(|s| s.name).unwrap_or_default()
        );
        play_song(&song.spotify_id, jam_id, pool, credentials).await?;
    }
    Ok(())
}

async fn handle_error(error: Error, close: bool, sender: &mpsc::Sender<ws::Message>) {
    eprintln!("Error: {:?}", error);

    if close {
        let close_frame = error.to_close_frame();
        if let Err(e) = sender.send(ws::Message::Close(Some(close_frame))).await {
            eprintln!("Error sending close frame: {:?}", e);
        }
    } else {
        let update = real_time::Update::new().error(error);
        let bin = match rmp_serde::to_vec(&update) {
            Ok(bin) => bin,
            Err(e) => {
                eprintln!("Error encoding error update: {:?}", e);
                return;
            }
        };
        if let Err(e) = sender.send(ws::Message::Binary(bin)).await {
            eprintln!("Error sending error update: {:?}", e);
        }
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
