use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ChatApiMessage {
    pub role: String,
    pub content: String,
}
