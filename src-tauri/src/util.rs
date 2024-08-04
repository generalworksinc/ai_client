use anyhow::Result;

use crate::API_KEY;
use async_openai::{config::OpenAIConfig, Client};

const OPENAI_THREAD_HEAD_WORD: &str = "thread_";
const OPENAI_ASSISTANT_HEAD_WORD: &str = "asst_";

pub fn create_client() -> Result<Client<OpenAIConfig>> {
    let api_key = API_KEY
        .read()
        .map_err(|e| anyhow::anyhow!("failed to read API_KEY: {:?}", e))?;
    Ok(Client::with_config(
        OpenAIConfig::new().with_api_key(api_key.clone()),
    ))
}

pub fn get_thread_and_assistant_id(id: &str) -> (String, String) {
    //thread_xxxxxxassistant_yyyyyy -> (xxxxxx, yyyyyy)
    if id.starts_with(OPENAI_THREAD_HEAD_WORD) {
        let parts: Vec<&str> = id.split(OPENAI_THREAD_HEAD_WORD).collect();
        let tmp_list = parts[1]
            .split(OPENAI_ASSISTANT_HEAD_WORD)
            .collect::<Vec<&str>>();
        let thread_id = tmp_list[0];
        let assistant_id = if tmp_list.len() >= 2 { tmp_list[1] } else { "" };

        (
            OPENAI_THREAD_HEAD_WORD.to_string() + thread_id,
            OPENAI_ASSISTANT_HEAD_WORD.to_string() + assistant_id,
        )
    } else {
        ("".to_string(), "".to_string())
    }
}
