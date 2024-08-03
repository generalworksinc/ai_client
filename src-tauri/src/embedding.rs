
use futures::StreamExt;

use serde::Deserialize;
use tauri::Window;

use crate::API_KEY;
use async_openai::{
    config::OpenAIConfig,
    types::CreateEmbeddingRequestArgs,
    Client,
};

#[tauri::command]
pub async fn embedding_test(
    awindow: Window,
    app_handle: tauri::AppHandle,
    params: String,
    timeout_sec: Option<u64>,
) -> Result<String, String> {
    #[derive(Deserialize)]
    struct PostData {
        message: Option<String>,
        // data: String,
        id: Option<String>,
    }
    // println!("call assistents_stream_test: {:#?}", params);
    let postData = serde_json::from_str::<PostData>(params.as_str()).unwrap();
    match embedding_example(postData.message.unwrap_or_default().as_str())
        .await
        .map_err(|e| format!("{:?}", e))
    {
        Ok(_) => Ok("テスト終了".to_string()),
        Err(err) => {
            // println!("Error: {:#?}", err);
            Err(err)
        }
    }
}

fn create_client() -> Client<OpenAIConfig> {
    Client::with_config(
        OpenAIConfig::new().with_api_key(API_KEY.read().map(|x| x.clone()).unwrap_or_default()),
    )
}

async fn embedding_example(question: &str) -> anyhow::Result<()> {
    let client = create_client();
    let request = CreateEmbeddingRequestArgs::default()
        .model("text-embedding-3-small")
        .input([
            "Why do programmers hate nature? It has too many bugs.",
            "Why was the computer cold? It left its Windows open.",
        ])
        .build()?;

    let response = client.embeddings().create(request).await?;

    for data in response.data {
        println!(
            "[{}]: has embedding of length {}",
            data.index,
            data.embedding.len()
        )
    }
    Ok(())
}
