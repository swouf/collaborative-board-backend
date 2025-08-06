use std::{collections::HashMap, sync::Arc};

use loro::LoroDoc;
use tokio::sync::{Mutex, broadcast};
use tracing::{event, Level};

use crate::models::doc_update::{DocUpdatePayload};

use super::message::ClientMessage;

#[derive(Debug)]
pub struct Room {
    pub sender: broadcast::Sender<(String, ClientMessage)>, // (user_id, content)
    pub state: LoroDoc, 
}

impl Room {
    pub fn new(updates: Vec<DocUpdatePayload>) -> Self {
        let (tx, _rx) = broadcast::channel(100);
        let doc = LoroDoc::new();
        match doc.import_batch(&updates) {
            Ok(_) => event!(Level::DEBUG, "Building document success."),
            Err(err) => event!(Level::ERROR, "Error building the document from stored updates.\n{}", err),
        }
        Self { sender: tx, state: doc }
    }
}

pub type Rooms = Arc<Mutex<HashMap<String, Room>>>;
