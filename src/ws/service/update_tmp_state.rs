use tracing::{Level, event};

use crate::ws::{
    message::{ServerMessage, UpdateTmpStateMessage},
    room::Rooms,
};

pub async fn handle(
    data: UpdateTmpStateMessage,
    rooms: &Rooms,
    conn_id: &str,
    current_room_id: &Option<String>,
) {
    event!(Level::DEBUG, "New TMP state update");
    if let Some(room_id) = &current_room_id {
        let rooms_lock = rooms.lock().await;

        let update_payload = data.payload.clone();
        let update_buffer: Vec<u8> = update_payload
            .chars()
            .map(|c| c as u32 as u8) // convert char -> u32 -> u8 (truncates like TypeScript)
            .collect();

        if let Some(room) = rooms_lock.get(room_id) {
            let msg = ServerMessage::UpdateTmpState(UpdateTmpStateMessage {
                payload: data.payload,
            });
            event!(Level::DEBUG, "Message ready to be sent.");
            let _ = room.sender.send((String::from(conn_id), msg));

            room.tmp_state.apply(&update_buffer);
        } else {
            event!(Level::WARN, "Unable to get room with id {}", room_id);
        }
    }
}
