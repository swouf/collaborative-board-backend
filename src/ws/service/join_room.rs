use std::collections::hash_map::Entry;

use crate::infra::db::schema::updates::{room_id, table};
use axum::extract::ws::Message;
use deadpool_diesel::postgres::Pool;
use diesel::prelude::*;
use tokio::sync::mpsc;
use tracing::{Level, event};

use crate::{
    models::doc_update::DocUpdate,
    ws::{
        message::{ClientMessage, JoinRoomMessage, ServerMessage, UpdateDocMessage},
        room::{Room, Rooms},
    },
};

async fn create_new_room(new_room_id: String, db_connection_pool: &Pool) -> Room {
    let conn = db_connection_pool.get().await.unwrap();
    let result = conn
        .interact(|conn| {
            table
                .filter(room_id.eq(new_room_id))
                .select(DocUpdate::as_select())
                .load(conn)
        })
        .await
        .map_err(|err| {
            event!(Level::ERROR, "DB interaction error: {}", err);
        })
        .unwrap();

    match result {
        Ok(doc_updates) => {
            Room::new(doc_updates.iter().map(|up| up.payload.clone()).collect())
        }
        Err(err) => {
            event!(Level::ERROR, "Impossible to create new room. Error {}", err);
            panic!("Oups");
        }
    }
}
// pub async fn handle(data: JoinRoomMessage, rooms: Rooms, tx: mpsc::Sender<Message>, current_room_id: Arc<Mutex<Option<String>>>) {
pub async fn handle(
    data: JoinRoomMessage,
    rooms: &Rooms,
    tx: &mpsc::Sender<Message>,
    current_room_id: &mut Option<String>,
    db_connection_pool: &Pool,
) {
    let mut rooms_lock = rooms.lock().await;
    let JoinRoomMessage { id, user_id } = data;
    let room_entry = rooms_lock.entry(id.clone());
    let room = match room_entry {
        Entry::Occupied(occupied_entry) => occupied_entry.into_mut(),
        Entry::Vacant(vacant_entry) => {
            vacant_entry.insert(create_new_room(id.clone(), db_connection_pool).await)
        }
    };
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
                            .send(Message::Text(serde_json::to_string(&msg).unwrap().into()))
                            .await;
                    }
                    ClientMessage::JoinRoom(_) => {
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
        .send(Message::Text(serde_json::to_string(&msg).unwrap().into()))
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
