use anyhow::Result;

use crate::API_KEY;
use async_openai::{config::OpenAIConfig, Client};

pub fn create_client() -> Result<Client<OpenAIConfig>> {
    let api_key = API_KEY
        .read()
        .map_err(|e| anyhow::anyhow!("failed to read API_KEY: {:?}", e))?;
    Ok(Client::with_config(
        OpenAIConfig::new().with_api_key(api_key.clone()),
    ))
}
