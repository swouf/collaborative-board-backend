use std::{collections::HashMap, sync::Arc};

use tokio::sync::{Mutex, broadcast};

use super::message::ClientMessage;

#[derive(Debug)]
pub struct Room {
    pub sender: broadcast::Sender<(String, ClientMessage)>, // (user_id, content)
}

impl Room {
    pub fn new() -> Self {
        let (tx, _rx) = broadcast::channel(100);
        Self { sender: tx }
    }
}

pub type Rooms = Arc<Mutex<HashMap<String, Room>>>;
