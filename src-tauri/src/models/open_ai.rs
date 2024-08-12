use crate::ChatApiMessageWithHtml;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default)]
pub struct OpenAIFileData {
    pub id: Option<String>,
    pub filename: Option<String>,
    pub purpose: Option<String>,
    pub bytes: Option<i64>,
    pub time: Option<String>,
    // {"id":"thread_aFRZqocRwwAJQ0wTBphELg1v","object":"thread","created_at":1723108843,"tool_resources":{},"metadata":{}}
}
#[derive(Deserialize, Serialize, Default)]
pub struct OpenAIVectorData {
    pub id: Option<String>,
    pub name: Option<String>,
    // pub purpose: Option<String>,
    pub usage_bytes: Option<i64>,
    pub created: Option<i64>,
    pub time: Option<String>,
    // {"id":"thread_aFRZqocRwwAJQ0wTBphELg1v","object":"thread","created_at":1723108843,"tool_resources":{},"metadata":{}}
}
