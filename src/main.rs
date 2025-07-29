mod ws;

use axum::{Router, routing::get};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use ws::{handler::ws_handler, room::Room};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let rooms = Arc::new(Mutex::new(HashMap::<String, Room>::new()));

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .with_state(rooms);

    // run it with hyper
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3433")
        .await
        .unwrap();

    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

// use axum::{
//     extract::{ws::Message, WebSocketUpgrade},
//     response::IntoResponse,
//     routing::get,
//     Router,
// };
// use futures::{SinkExt, StreamExt};
// use serde::{Deserialize, Serialize};
// use tokio_tungstenite::tungstenite::client;
// use tracing::{debug, error, event, instrument, Level};
// use std::{
//     collections::HashMap,
//     net::SocketAddr,
//     sync::Arc,
// };
// use tokio::sync::{Mutex, broadcast, mpsc};
// use uuid::Uuid;

// use tower_http::{
//     services::ServeDir,
//     trace::{DefaultMakeSpan, TraceLayer},
// };

// use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// // Shared state: rooms map room_id -> broadcaster
// type RoomId = String;
// type Rooms = Arc<Mutex<HashMap<RoomId, Room>>>;

// #[tokio::main]
// async fn main() {
//     tracing_subscriber::registry()
//         .with(
//             tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
//                 format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
//             }),
//         )
//         .with(tracing_subscriber::fmt::layer())
//         .init();

//     let rooms: Rooms = Arc::new(Mutex::new(HashMap::new()));

//     let app = Router::new()
//         .route("/ws", get(handle_ws_upgrade))
//         .with_state(rooms);

//     // run it with hyper
//     let listener = tokio::net::TcpListener::bind("127.0.0.1:3433")
//         .await
//         .unwrap();

//     tracing::debug!("listening on {}", listener.local_addr().unwrap());
//     axum::serve(
//         listener,
//         app.into_make_service_with_connect_info::<SocketAddr>(),
//     )
//     .await
//     .unwrap();
// }

// async fn handle_ws_upgrade(
//     ws: WebSocketUpgrade,
//     axum::extract::State(rooms): axum::extract::State<Rooms>,
// ) -> impl IntoResponse {
//     ws.on_upgrade(|socket| handle_socket(socket, rooms))
// }

// #[derive(Serialize, Deserialize)]
// #[serde(tag = "type", rename_all = "UPPERCASE")]
// enum ClientMessage {
//     JoinRoom { id: String },
//     Message { content: String }
// }

// #[derive(Serialize)]
// struct ServerMessage {
//     user_id: String,
//     content: String,
// }

// #[derive(Debug)]
// struct Room {
//     sender: broadcast::Sender<(String, String)>, // (user_id, content)
// }

// #[instrument]
// async fn handle_socket(stream: axum::extract::ws::WebSocket, rooms: Rooms) {
//     let (ws_sender, mut ws_receiver) = stream.split();
//     let user_id = Uuid::new_v4().to_string();
//     let mut maybe_room_id = None;

//     // Set up a channel to send messages to the WebSocket sender task
//     let (tx, mut rx): (mpsc::Sender<Message>, mpsc::Receiver<Message>) = mpsc::channel(32);

//     // Spawn a task to forward messages to the client over the WebSocket
//     tokio::spawn(async move {
//         let mut ws_sender = ws_sender;
//         while let Some(msg) = rx.recv().await {
//             if ws_sender.send(msg).await.is_err() {
//                 break;
//             }
//         }
//     });

//     let mut room_rx: Option<broadcast::Receiver<(String, String)>> = None;

//     while let Some(Ok(msg)) = ws_receiver.next().await {
//         if let Message::Text(text) = msg {
//             debug!("New message: {}", &text);
//             match serde_json::from_str::<ClientMessage>(&text) {
//                 Ok(client_msg) => {
//                     match client_msg {
//                     ClientMessage::JoinRoom { id } => {
//                         debug!("Joining room...");
//                         let mut rooms_guard = rooms.lock().await;

//                         let room = rooms_guard
//                             .entry(id.clone())
//                             .or_insert_with(|| {
//                                 let (tx, _rx) = broadcast::channel(100);
//                                 Room { sender: tx }
//                             });

//                         let mut rx_clone = room.sender.subscribe();
//                         maybe_room_id = Some(id.clone());

//                         let tx_clone = tx.clone();
//                         let user_id_clone = user_id.clone();

//                         // Spawn task to listen for room broadcasts and forward them to this user
//                         tokio::spawn(async move {
//                             while let Ok((uid, content)) = rx_clone.recv().await {
//                                 if uid != user_id_clone {
//                                     let msg = ServerMessage {
//                                         user_id: uid,
//                                         content,
//                                     };
//                                     if tx_clone
//                                         .send(Message::Text(serde_json::to_string(&msg).unwrap()))
//                                         .await
//                                         .is_err()
//                                     {
//                                         break;
//                                     }
//                                 }
//                             }
//                         });

//                         let ok = String::from("Ok");
//                         if tx.send(Message::Text(ok)).await.is_err() {
//                             error!("Unable to confirm joining room.");
//                         }
//                     }
//                     ClientMessage::Message { content } => {
//                         if let Some(room_id) = &maybe_room_id {
//                             let rooms_guard = rooms.lock().await;
//                             if let Some(room) = rooms_guard.get(room_id) {
//                                 let _ = room.sender.send((user_id.clone(), content));
//                             }
//                         }
//                     }
//                 }
//             }
//                 Err(_) => {
//                     let example_join = ClientMessage::JoinRoom { id: String::from("sample-id") };
//                     let r = serde_json::to_string(&example_join);
//                     match r {
//                         Ok(ex_join_str)=>{
//                             if tx.send(Message::Text(ex_join_str)).await.is_err() {
//                                 error!("Unable to say oups.");
//                             }
//                         }
//                         Err(_) => {
//                             error!("Impossible to serialize");
//                         },
//                     }
//                 }
//             }
//             // if let Ok(client_msg) =  {
//             //     debug!("Message is valid.");
//             //     match client_msg
//             // }
//         }
//     }

//     // Connection closed, everything shuts down automatically
// }
