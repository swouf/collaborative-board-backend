use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[serde(rename_all = "snake_case")]
pub enum AIQueries {
    GetFeedback,
    GetComment,
}
