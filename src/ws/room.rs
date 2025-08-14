use std::{collections::HashMap, sync::Arc};

use loro::{LoroDoc, awareness::EphemeralStore};
use tokio::sync::{Mutex, broadcast};
use tracing::{Level, event};

use crate::models::doc_update::DocUpdatePayload;

use super::message::ServerMessage;

#[derive(Debug)]
pub struct Room {
    pub sender: broadcast::Sender<(String, ServerMessage)>, // (user_id, content)
    pub state: LoroDoc,
    pub tmp_state: EphemeralStore,
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

        Self {
            sender: tx,
            state: doc,
            tmp_state,
        }
    }
}

pub type Rooms = Arc<Mutex<HashMap<String, Room>>>;
