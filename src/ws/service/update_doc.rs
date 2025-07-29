use tracing::{Level, event, instrument};

use crate::ws::{
    message::{ClientMessage, UpdateDocMessage},
    room::Rooms,
};

#[instrument]
pub async fn handle(
    data: UpdateDocMessage,
    rooms: &Rooms,
    conn_id: &String,
    current_room_id: &Option<String>,
) {
    event!(
        Level::DEBUG,
        "New update doc message with data {}",
        data.payload
    );
    if let Some(room_id) = &current_room_id {
        let rooms_lock = rooms.lock().await;
        if let Some(room) = rooms_lock.get(room_id) {
            let msg = ClientMessage::UpdateDoc(UpdateDocMessage {
                payload: data.payload,
            });
            event!(Level::DEBUG, "Message ready to be sent.");
            let _ = room.sender.send((conn_id.clone(), msg));
        } else {
            event!(Level::WARN, "Unable to get room with id {}", room_id);
        }
    }
}
