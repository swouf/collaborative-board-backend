use axum::extract::ws::Message;
use loro::{LoroValue, loro_value};
use tokio::sync::mpsc;
use tracing::{Level, event};

use crate::{
    ai::get_feedback::get_feedback,
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

        if let Some(room) = rooms_lock.get(room_id) {
            let responses = room.state.get_list("responses");
            let first_response = responses
                .get(0)
                .unwrap()
                .into_value()
                .unwrap()
                .into_map()
                .unwrap(); // TODO: Handle error instead of unwrap
            // print!("First response: {:#?}", first_response);

            // This is a test.

            let first_response_str = first_response
                .clone()
                .get("response")
                .unwrap()
                .clone()
                .into_string()
                .unwrap()
                .unwrap();

            // first_response_str.

            match get_feedback(&first_response_str).await {
                Some(response) => {
                    let response_payload = serde_json::to_string(&response).unwrap();
                    let msg = ServerMessage::Confirm {
                        message_type: String::from("query_ai"),
                        message: Some(response_payload),
                    };
                    let mut r_clone = first_response.clone();
                    let updated_response = r_clone.make_mut();

                    updated_response.insert(String::from("feedback"), loro_value!(response));

                    event!(
                        Level::DEBUG,
                        "Here is the updated response: {:#?}",
                        updated_response
                    );

                    responses.delete(0, 1).unwrap();
                    responses.insert(0, LoroValue::Map(r_clone)).unwrap();

                    event!(Level::DEBUG, "Responses updated:\n{:#?}", responses);

                    room.state.commit();

                    tx.send(Message::Text(serde_json::to_string(&msg).unwrap().into()))
                        .await
                        .unwrap();
                }
                None => {
                    let msg = ServerMessage::Error {
                        message: String::from("Shit happened."),
                    };
                    tx.send(Message::Text(serde_json::to_string(&msg).unwrap().into()))
                        .await
                        .unwrap();
                }
            }

            // let msg = ServerMessage::UpdateDoc(UpdateDocMessage {
            //     payload: data.payload,
            // });
            // event!(Level::DEBUG, "Message ready to be sent.");
            // let _ = room.sender.send((String::from(conn_id), msg));
        } else {
            event!(Level::WARN, "Unable to get room with id {}", room_id);
        };
    }
}
