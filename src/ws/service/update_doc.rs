use deadpool_diesel::mysql::Pool;
use tracing::{Level, event};
use diesel::{insert_into, RunQueryDsl};

use crate::{infra::db::schema::updates::dsl::updates, models::doc_update::NewDocUpdate, ws::{message::{ClientMessage, UpdateDocMessage}, room::Rooms}};

pub async fn handle(
    data: UpdateDocMessage,
    rooms: &Rooms,
    db_connection_pool: &Pool,
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
                payload: data.payload.clone(),
            });
            event!(Level::DEBUG, "Message ready to be sent.");
            let _ = room.sender.send((conn_id.clone(), msg));
        } else {
            event!(Level::WARN, "Unable to get room with id {}", room_id);
        }

        let update_payload = data.payload;
        let update_buffer: Vec<u8> = update_payload
            .chars()
            .map(|c| c as u32 as u8)  // convert char -> u32 -> u8 (truncates like TypeScript)
            .collect();

        let room_id_copy = room_id.clone();
        let insertable_doc_update = NewDocUpdate {
            room_id: room_id_copy,
            payload: update_buffer,
        };
        
        let conn = db_connection_pool.get().await.unwrap();
        let _ = conn
            .interact(|conn| {
                insert_into(updates)
                    .values(insertable_doc_update)
                    .execute(conn)
            })
            .await.map_err(
                |err| {
                    event!(Level::ERROR, "DB interaction error: {}", err);
                }
            ).unwrap();
    }
}
