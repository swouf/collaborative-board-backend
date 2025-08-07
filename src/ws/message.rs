use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
pub struct JoinRoomMessage {
    pub id: String,
    pub user_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
pub struct UpdateDocMessage {
    pub payload: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
pub struct GetDocMessage {
    pub version_vector: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, TS)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
#[ts(export)]
pub enum ClientMessage {
    JoinRoom(JoinRoomMessage),
    UpdateDoc(UpdateDocMessage),
    GetDoc(GetDocMessage),
}

#[derive(Serialize, TS)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
#[ts(export)]
pub enum ServerMessage {
    Confirm {
        message_type: String,
        message: Option<String>,
    },
    UpdateDoc(UpdateDocMessage),
    Error {
        message: String,
    },
}
