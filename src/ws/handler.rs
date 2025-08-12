use axum::extract::ws::{Message, WebSocket};
use axum::{
    extract::{State, WebSocketUpgrade},
    response::IntoResponse,
};
use deadpool_diesel::postgres::Pool;
use futures::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tracing::{Level, error, event};
use uuid::Uuid;

use crate::AppState;
use crate::ws::message::{ClientMessage, ServerMessage};
use crate::ws::room::Rooms;
use crate::ws::service::get_doc;

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

    // let current_room_id: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let mut current_room_id: Option<String> = None;

    // You transmit your message via tx, and receive it via rx.
    let (tx, mut rx): (mpsc::Sender<Message>, mpsc::Receiver<Message>) = mpsc::channel(32);

    // Forwarding task
    tokio::spawn(async move {
        let mut ws_sender = ws_sender;
        while let Some(msg) = rx.recv().await {
            if ws_sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    while let Some(Ok(msg)) = ws_receiver.next().await {
        event!(
            Level::DEBUG,
            "New message received on connection {}",
            conn_id
        );
        if let Message::Text(text) = msg {
            match serde_json::from_str::<ClientMessage>(&text) {
                // if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                Ok(client_msg) => match client_msg {
                    ClientMessage::JoinRoom(data) => {
                        join_room::handle(
                            data,
                            &rooms,
                            &tx,
                            &mut current_room_id,
                            &db_connection_pool,
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
                        update_tmp_state::handle(
                            data,
                            &rooms,
                            &conn_id,
                            &current_room_id,
                        )
                        .await
                    }
                    ClientMessage::GetDoc(_) => {
                        get_doc::handle(&rooms, &tx, &current_room_id).await
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
}
