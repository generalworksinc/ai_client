use crate::ChatApiMessageWithHtml;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ChatApiMessage {
    pub role: String,
    pub content: String,
}
impl ChatApiMessage {
    pub fn convert_with_html(&self) -> ChatApiMessageWithHtml {
        ChatApiMessageWithHtml {
            role: self.role.clone(),
            content: self.content.clone(),
            content_html: None,
        }
    }
}
