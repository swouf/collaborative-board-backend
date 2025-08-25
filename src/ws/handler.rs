use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use axum::{
    extract::{State, WebSocketUpgrade},
    response::IntoResponse,
};
use deadpool_diesel::postgres::Pool;
use futures::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio::time::timeout;
use tracing::{Level, error, event};
use uuid::Uuid;

use crate::AppState;
use crate::constants::TIMEOUT_DURATION;
use crate::ws::message::{ClientMessage, ServerMessage};
use crate::ws::room::Rooms;
use crate::ws::service::{get_doc, query_ai};

use super::message::JoinRoomMessage;
use super::service::{join_room, update_doc, update_tmp_state};

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(AppState {
        rooms,
        db_connection_pool,
    }): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, rooms, db_connection_pool))
}

async fn handle_socket(socket: WebSocket, rooms: Rooms, db_connection_pool: Pool) {
    let (ws_sender, mut ws_receiver) = socket.split();
    let conn_id = Uuid::new_v4().to_string();
    let mut keep_alive = Arc::new(true);

    // let current_room_id: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let mut current_room_id: Option<String> = None;

    // You transmit your message via tx, and receive it via rx.
    let (tx, mut rx): (mpsc::Sender<Message>, mpsc::Receiver<Message>) = mpsc::channel(32);

    let keep_alive_forward = Arc::downgrade(&keep_alive);
    let debug_conn_id = conn_id.clone();
    // Forwarding task
    tokio::spawn(async move {
        let mut ws_sender = ws_sender;
        while keep_alive_forward.upgrade().is_some() {
            if let Ok(Some(msg)) = timeout(TIMEOUT_DURATION, rx.recv()).await {
                if ws_sender.send(msg).await.is_err() {
                    break;
                }
            }
        }
        event!(
            Level::DEBUG,
            "Forwarding task for connection {} exiting...",
            debug_conn_id
        );
    });

    // while let Ok(Some(frame)) = timeout(TIMEOUT_DURATION, ws_receiver.next()).await {
    while let Some(frame) = ws_receiver.next().await {
        event!(Level::DEBUG, "New frame received on connection {}", conn_id);
        match frame {
            Ok(msg) => {
                if let Message::Text(text) = msg {
                    match serde_json::from_str::<ClientMessage>(&text) {
                        // if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                        Ok(client_msg) => match client_msg {
                            ClientMessage::JoinRoom(data) => {
                                let keep_alive_receiver = Arc::downgrade(&keep_alive);
                                join_room::handle(
                                    data,
                                    &rooms,
                                    &tx,
                                    &mut current_room_id,
                                    &db_connection_pool,
                                    keep_alive_receiver,
                                )
                                .await
                            }
                            ClientMessage::UpdateDoc(data) => {
                                update_doc::handle(
                                    data,
                                    &rooms,
                                    &db_connection_pool,
                                    &conn_id,
                                    &current_room_id,
                                )
                                .await
                            }
                            ClientMessage::UpdateTmpState(data) => {
                                update_tmp_state::handle(data, &rooms, &conn_id, &current_room_id)
                                    .await
                            }
                            ClientMessage::GetDoc(_) => {
                                get_doc::handle(&rooms, &tx, &current_room_id).await
                            }
                            ClientMessage::QueryAI(data) => {
                                query_ai::handle(data, &rooms, &tx, &current_room_id).await
                            }
                        },
                        Err(_) => {
                            let example_join = ClientMessage::JoinRoom(JoinRoomMessage {
                                id: String::from("room-id"),
                                user_id: String::from("user-id"),
                            });
                            let r = serde_json::to_string(&example_join);
                            if let Ok(ex_str) = r {
                                let err_msg = ServerMessage::Error {
                                    message: format!(
                                        "I could not interpret your message. Please, join a room.\nExample:\n{ex_str}"
                                    ),
                                };
                                if let Ok(err_msg_str) = serde_json::to_string(&err_msg) {
                                    if tx.send(Message::Text(err_msg_str.into())).await.is_err() {
                                        error!("Error when sending error message.")
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Err(err) => {
                // Frame reception error
                event!(
                    Level::INFO,
                    "Error in receiving frame on connection {}.\nMessage: {}",
                    conn_id,
                    err
                );
            }
        }
    }
    event!(Level::INFO, "Client connection {} timed out.", conn_id);
    let keep_alive_mut = Arc::make_mut(&mut keep_alive);
    *keep_alive_mut = false;
}
