use axum::extract::ws::Message;
use tokio::sync::mpsc;
use tracing::{Level, event};

use crate::{
    ai::{get_feedback::get_feedback, queries::AIQueries},
    ws::{
        message::{QueryAIMessage, ServerMessage},
        room::Rooms,
    },
};

pub async fn handle(
    data: QueryAIMessage,
    rooms: &Rooms,
    tx: &mpsc::Sender<Message>,
    // conn_id: &str,
    current_room_id: &Option<String>,
) {
    event!(Level::DEBUG, "{:#?} from AI", data.verb);

    if let Some(room_id) = &current_room_id {
        let rooms_lock = rooms.lock().await;
        let msg: ServerMessage;

        if let Some(room) = rooms_lock.get(room_id) {
            let response = match data.verb {
                AIQueries::GetFeedback => get_feedback(data, room).await,
                AIQueries::GetComment => todo!(),
            };

            match response {
                Ok(payload) => {
                    msg = ServerMessage::Confirm {
                        message_type: String::from("query_ai"),
                        message: Some(payload),
                    };
                }
                Err(err) => {
                    msg = ServerMessage::Error {
                        message: format!("Error happened when handling the request.\n{err}"),
                    };
                }
            }

            tx.send(Message::Text(serde_json::to_string(&msg).unwrap().into()))
                .await
                .unwrap();
        } else {
            event!(Level::WARN, "Unable to get room with id {}", room_id);
        };
    }
}
