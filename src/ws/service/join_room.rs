use axum::extract::ws::Message;
use tokio::sync::mpsc;
use tracing::{Level, event};

use crate::ws::{
    message::{ClientMessage, JoinRoomMessage, ServerMessage, UpdateDocMessage},
    room::{Room, Rooms},
};

// pub async fn handle(data: JoinRoomMessage, rooms: Rooms, tx: mpsc::Sender<Message>, current_room_id: Arc<Mutex<Option<String>>>) {
pub async fn handle(
    data: JoinRoomMessage,
    rooms: &Rooms,
    tx: &mpsc::Sender<Message>,
    current_room_id: &mut Option<String>,
) {
    let mut rooms_lock = rooms.lock().await;
    let JoinRoomMessage { id, user_id } = data;
    let room = rooms_lock.entry(id.clone()).or_insert_with(Room::new);

    let mut room_rx = room.sender.subscribe();

    // let current_room_id_lock = current_room_id.lock().await;
    *current_room_id = Some(id.clone());
    event!(Level::DEBUG, "current_room_id is now {:?}", current_room_id);

    let tx_clone = tx.clone();
    let user_id_clone = user_id.clone();

    // Subscribe task
    tokio::spawn(async move {
        while let Ok((uid, content)) = room_rx.recv().await {
            if uid != user_id_clone {
                match content {
                    ClientMessage::UpdateDoc(data) => {
                        let msg = ServerMessage::UpdateDoc(UpdateDocMessage {
                            payload: data.payload,
                        });
                        let _ = tx_clone
                            .send(Message::Text(serde_json::to_string(&msg).unwrap()))
                            .await;
                    }
                    ClientMessage::JoinRoom(join_room_message) => {
                        todo!("A join room message shouldn't get here.")
                    }
                }
            }
        }
    });

    let msg = ServerMessage::Confirm {
        message_type: String::from("join_room"),
        message: None,
    };
    let confirm = tx
        .send(Message::Text(serde_json::to_string(&msg).unwrap()))
        .await;
    match confirm {
        Ok(_) => {
            event!(Level::DEBUG, "Confirm sent.")
        }
        Err(err) => {
            event!(
                Level::ERROR,
                "Error when sending confirmation.\n{}",
                err.to_string()
            )
        }
    }
}
