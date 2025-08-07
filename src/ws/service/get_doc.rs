use axum::extract::ws::Message;
use loro::ExportMode;
use tokio::sync::mpsc;
use tracing::{Level, event};

use crate::ws::{
    message::{ServerMessage, UpdateDocMessage},
    room::Rooms,
};

pub async fn handle(rooms: &Rooms, tx: &mpsc::Sender<Message>, current_room_id: &Option<String>) {
    event!(Level::DEBUG, "Getting doc state.",);
    if let Some(room_id) = &current_room_id {
        let rooms_lock = rooms.lock().await;

        let msg = if let Some(room) = rooms_lock.get(room_id) {
            let res = room.state.export(ExportMode::Snapshot);

            match res {
                Ok(snapshot) => {
                    let p = snapshot.iter().map(|&b| b as char).collect();
                    ServerMessage::UpdateDoc(UpdateDocMessage { payload: p })
                }
                Err(err) => {
                    let notification_text = "Error getting document's snapshot.";
                    event!(Level::ERROR, "{}\n{}", notification_text, err);
                    ServerMessage::Error {
                        message: notification_text.to_string(),
                    }
                }
            }
        } else {
            let error_txt = format!("Unable to get room with id {room_id}");
            event!(Level::WARN, error_txt);
            ServerMessage::Error { message: error_txt }
        };

        let response_sent = tx
            .send(Message::Text(serde_json::to_string(&msg).unwrap().into()))
            .await;
        match response_sent {
            Ok(_) => {
                event!(Level::DEBUG, "Confirm sent.");
            }
            Err(err) => {
                event!(
                    Level::ERROR,
                    "Error when sending confirmation.\n{}",
                    err.to_string()
                );
            }
        }
    }
}
