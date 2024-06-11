use crate::general_types::*;
use leptos::*;
#[cfg(feature="ssr")]
use axum::{
    extract::{ws::*, State, Query},
    response::Response,
};
#[cfg(feature="ssr")]
use futures_util::{
    sink::SinkExt,
    stream::{SplitSink, SplitStream, StreamExt},
};

#[derive(Debug, serde::Deserialize)]
pub struct JamId(pub String);


#[cfg(feature="ssr")]
pub async fn get_users_handler(ws: WebSocketUpgrade, Query(jam_id):Query<JamId>, State(state):State<AppState>) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state, jam_id.0))
}

#[cfg(feature="ssr")]
async fn handle_socket(socket: WebSocket, app_state: AppState, jam_id: String) {
    let (sender, _) = socket.split();

    tokio::spawn(write(sender, jam_id.clone(), app_state.clone()));
}

#[cfg(feature="ssr")]
async fn write(mut sender: SplitSink<WebSocket, Message>, jam_id: String, app_state: AppState) {
    let pool=app_state.db.pool;
    
    let listener=sqlx::postgres::PgListener::connect_with(&pool).await;
    let mut listener = match listener {
        Ok(listener) => listener,
        Err(e) => {
            eprintln!("Error connecting to listener: {:?}", e);
            return;
        }
    };
    
    if let Err(e) = listener.listen(format!("{}users", jam_id).as_str()).await {
        eprintln!("Error listening to channel: {:?}", e);
        return;
    }
    
    
    loop{
        let notification=match listener.try_recv().await{
            Ok(Some(notification))=>notification,
            Err(e)=>{
                eprintln!("Error receiving notification: {:?}", e);
                continue;
            },
            Ok(None)=>{
                eprint!("disconnected from pool, reconnecting on next iteration");
                continue;
            }
        };

        println!("Received notification: {:#?}", notification);

        let result=match sqlx::query_as!{User, "SELECT * FROM users WHERE jam_id=$1", jam_id.clone()}
            .fetch_all(& pool).await {
                Ok(result)=>result,
                Err(e)=>{
                    eprintln!("Error fetching users: {:?}", e);
                    continue;
                }
            };
        
        let json=match serde_json::to_string(&result){
            Ok(json)=>json,
            Err(e)=>{
                eprintln!("Error serializing users: {:?}", e);
                continue;
            }
        };
        match sender.send(Message::Text(json)).await {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Error sending message: {:?}", e);
                continue;
            }
        }


    }
}

#[component]
pub fn UsersBar(
    jam_id: MaybeSignal<String>,
    #[prop(optional_no_strip)] host_id: Option<MaybeSignal<String>>,
) -> impl IntoView {
    view! {}
}
