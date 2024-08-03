use std::error::Error;
use tauri::{CustomMenuItem, Menu, MenuItem, Submenu};
use tauri::{Manager, Window, WindowUrl};
use serde::Deserialize;

use async_openai::{
    config::OpenAIConfig,
    types::{
        CreateSpeechRequestArgs, SpeechModel, Voice,
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
use futures::StreamExt;
use crate::API_KEY;


#[tauri::command]
pub async fn assistants_audio_speech_test(awindow: Window,
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
    match audio_speech_example(postData.message.unwrap_or_default().as_str()).await.map_err(|e| format!("{:?}", e)) {
        Ok(_) => {
            Ok("テスト終了".to_string())
        }
        Err(err) => {
            println!("Error: {:#?}", err);
            Err(err)
        }
    }
}

fn create_client() -> Client<OpenAIConfig> {
    Client::with_config(OpenAIConfig::new().with_api_key(API_KEY.read().map(|x| x.clone()).unwrap_or_default()))
}
async fn audio_speech_example(question: &str) -> anyhow::Result<()>{
    let client = create_client();
    let request = CreateSpeechRequestArgs::default()
        // .input("Today is a wonderful day to build something people love!")
        .input("CodeZine（コードジン）は、ITエンジニアの成長や課題解決に役立つ記事やイベントレポート、ニュースなどを提供する情報サイトです。ChatGPTや生成AI、Flutter、Pythonなどの注目のテーマや技術を紹介しています。")
        .voice(Voice::Alloy)
        .model(SpeechModel::Tts1)
        .build()?;

    let response = client.audio().speech(request).await?;

    // response.save("./data/audio.mp3").await?;
    response.save("./data/codezine.mp3").await?;
    
    Ok(())
}
