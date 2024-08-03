use std::error::Error;
use tauri::{CustomMenuItem, Menu, MenuItem, Submenu};
use tauri::{Manager, Window, WindowUrl};
use serde::Deserialize;

use async_openai::{
    config::OpenAIConfig,
    types::{
        AudioResponseFormat, CreateTranscriptionRequestArgs, TimestampGranularity,
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
use futures::StreamExt;
use crate::API_KEY;


#[tauri::command]
pub async fn assistants_audio_transcribe_test(awindow: Window,
    app_handle: tauri::AppHandle,
    params: String,
    timeout_sec: Option<u64>,) -> Result<String, String> {
    #[derive(Deserialize)]
    struct PostData {
        message: Option<String>,
        // data: String,
        id: Option<String>,
    }
    println!("call assistents_stream_test: {:#?}", params);
    let postData = serde_json::from_str::<PostData>(params.as_str()).unwrap();
    match audio_transcribe_example(postData.message.unwrap_or_default().as_str()).await.map_err(|e| format!("{:?}", e)) {
        Ok(_) => {
            Ok("テスト終了".to_string())
        }
        Err(err) => {
            Err(err)
        }
    }
}

fn create_client() -> Client<OpenAIConfig> {
    Client::with_config(OpenAIConfig::new().with_api_key(API_KEY.read().map(|x| x.clone()).unwrap_or_default()))
}
async fn audio_transcribe_example(question: &str) -> anyhow::Result<()>{
    transcribe_json().await?;
    transcribe_verbose_json().await?;
    transcribe_srt().await?;

    // //create a client
    // let client = Client::with_config(OpenAIConfig::new().with_api_key(API_KEY.read().map(|x| x.clone()).unwrap_or_default()));
    
    Ok(())
}


async fn transcribe_json() -> Result<(), anyhow::Error> {
    let client = create_client();
    // Credits and Source for audio: https://www.youtube.com/watch?v=oQnDVqGIv4s
    let request = CreateTranscriptionRequestArgs::default()
        .file(
            "./audio/A Message From Sir David Attenborough A Perfect Planet BBC Earth_320kbps.mp3",
        )
        .model("whisper-1")
        .response_format(AudioResponseFormat::Json)
        .build()?;

    let response = client.audio().transcribe(request).await?;
    println!("{}", response.text);
    Ok(())
}

async fn transcribe_verbose_json() -> Result<(), anyhow::Error> {
    let client = create_client();
    let request = CreateTranscriptionRequestArgs::default()
        .file(
            "./audio/A Message From Sir David Attenborough A Perfect Planet BBC Earth_320kbps.mp3",
        )
        .model("whisper-1")
        .response_format(AudioResponseFormat::VerboseJson)
        .timestamp_granularities(vec![
            TimestampGranularity::Word,
            TimestampGranularity::Segment,
        ])
        .build()?;

    let response = client.audio().transcribe_verbose_json(request).await?;

    println!("{}", response.text);
    if let Some(words) = &response.words {
        println!("- {} words", words.len());
    }
    if let Some(segments) = &response.segments {
        println!("- {} segments", segments.len());
    }

    Ok(())
}

async fn transcribe_srt() -> Result<(), anyhow::Error> {
    let client = create_client();
    let request = CreateTranscriptionRequestArgs::default()
        .file(
            "./audio/A Message From Sir David Attenborough A Perfect Planet BBC Earth_320kbps.mp3",
        )
        .model("whisper-1")
        .response_format(AudioResponseFormat::Srt)
        .build()?;

    let response = client.audio().transcribe_raw(request).await?;
    println!("{}", String::from_utf8_lossy(response.as_ref()));
    Ok(())
}