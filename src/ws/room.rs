use std::{collections::HashMap, sync::Arc};

use loro::{LoroDoc, Subscription, awareness::EphemeralStore};
use tokio::sync::{Mutex, broadcast};
use tracing::{Level, event};

use crate::{models::doc_update::DocUpdatePayload, ws::message::UpdateDocMessage};

use super::message::ServerMessage;

#[derive(Debug)]
pub struct Room {
    pub sender: broadcast::Sender<(String, ServerMessage)>, // (user_id, content)
    pub state: LoroDoc,
    pub tmp_state: EphemeralStore,
    subscription: Subscription,
}

impl Room {
    pub fn new(updates: Vec<DocUpdatePayload>) -> Self {
        let (tx, _rx) = broadcast::channel(100);
        let doc = LoroDoc::new();
        let tmp_state = EphemeralStore::new(30000); // TODO: Factor out.
        match doc.import_batch(&updates) {
            Ok(_) => event!(Level::DEBUG, "Building document success."),
            Err(err) => event!(
                Level::ERROR,
                "Error building the document from stored updates.\n{}",
                err
            ),
        }

        let tx_for_updates = tx.clone();

        let subscription =
            doc.subscribe_local_update(Box::new(move |local_update: &Vec<u8>| -> bool {
                let update_str = local_update.iter().map(|&b| b as char).collect();
                event!(Level::DEBUG, "New local update: {}", update_str);
                let msg = ServerMessage::UpdateDoc(UpdateDocMessage {
                    payload: update_str,
                });
                match tx_for_updates.send((0.to_string(), msg)) {
                    Ok(_) => event!(Level::DEBUG, "State locally updated."),
                    Err(_) => event!(Level::ERROR, "Unable to send local state update."),
                }
                true
            }));

        Self {
            sender: tx,
            state: doc,
            tmp_state,
            subscription,
        }
    }
}

pub type Rooms = Arc<Mutex<HashMap<String, Room>>>;
