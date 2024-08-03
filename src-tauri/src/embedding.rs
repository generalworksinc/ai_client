use std::collections::HashMap;
use std::io::{stdout, Write};
use anyhow::Context;

use futures::StreamExt;
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use serde_json::{json, Value};

use tauri::{CustomMenuItem, Menu, MenuItem, Submenu};
use tauri::{Manager, Window, WindowUrl};
use serde::Deserialize;

use async_openai::{
    config::OpenAIConfig,
    types::{
        CreateEmbeddingRequestArgs,
        ChatCompletionMessageToolCall, ChatCompletionRequestAssistantMessageArgs,
    ChatCompletionRequestMessage, ChatCompletionRequestToolMessageArgs,
    ChatCompletionRequestUserMessageArgs, ChatCompletionToolArgs, ChatCompletionToolType,
    FunctionObjectArgs,
        CreateSpeechRequestArgs, SpeechModel, Voice,
        ChatCompletionRequestMessageContentPartImageArgs,
        ChatCompletionRequestMessageContentPartTextArgs, 
        CreateChatCompletionRequestArgs, ImageDetail, ImageUrlArgs,
        AudioResponseFormat, CreateTranscriptionRequestArgs, TimestampGranularity,
        CreateTranslationRequestArgs,
        AssistantStreamEvent, CreateAssistantRequestArgs, CreateMessageRequest, CreateRunRequest,
        CreateThreadRequest, FunctionObject, MessageDeltaContent, MessageRole, RunObject,
        SubmitToolOutputsRunRequest, ToolsOutputs,
        CreateMessageRequestArgs, CreateRunRequestArgs,
        CreateThreadRequestArgs,
        AssistantToolFileSearchResources, AssistantToolsFileSearch, 
        CreateFileRequest,
        CreateVectorStoreRequest, FilePurpose, MessageAttachment, MessageAttachmentTool,
        MessageContent,  ModifyAssistantRequest, RunStatus,
        AssistantToolCodeInterpreterResources, AssistantTools, MessageContentTextAnnotations, 
    },
    Client,
};
use crate::API_KEY;


#[tauri::command]
pub async fn embedding_test(awindow: Window,
    app_handle: tauri::AppHandle,
    params: String,
    timeout_sec: Option<u64>,) -> Result<String, String> {
    #[derive(Deserialize)]
    struct PostData {
        message: Option<String>,
        // data: String,
        id: Option<String>,
    }
    // println!("call assistents_stream_test: {:#?}", params);
    let postData = serde_json::from_str::<PostData>(params.as_str()).unwrap();
    match embedding_example(postData.message.unwrap_or_default().as_str()).await.map_err(|e| format!("{:?}", e)) {
        Ok(_) => {
            Ok("テスト終了".to_string())
        }
        Err(err) => {
            // println!("Error: {:#?}", err);
            Err(err)
        }
    }
}

fn create_client() -> Client<OpenAIConfig> {
    Client::with_config(OpenAIConfig::new().with_api_key(API_KEY.read().map(|x| x.clone()).unwrap_or_default()))
}

async fn embedding_example(question: &str) -> anyhow::Result<()>{
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