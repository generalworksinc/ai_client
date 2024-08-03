use serde::Deserialize;
use std::error::Error;
use tauri::{CustomMenuItem, Menu, MenuItem, Submenu};
use tauri::{Manager, Window, WindowUrl};

use crate::API_KEY;
use async_openai::{
    config::OpenAIConfig,
    types::{
        AssistantStreamEvent, AssistantToolCodeInterpreterResources,
        AssistantToolFileSearchResources, AssistantTools, AssistantToolsFileSearch,
        AudioResponseFormat, ChatCompletionRequestMessageContentPartImageArgs,
        ChatCompletionRequestMessageContentPartTextArgs, ChatCompletionRequestUserMessageArgs,
        CreateAssistantRequestArgs, CreateChatCompletionRequestArgs, CreateFileRequest,
        CreateMessageRequest, CreateMessageRequestArgs, CreateRunRequest, CreateRunRequestArgs,
        CreateSpeechRequestArgs, CreateThreadRequest, CreateThreadRequestArgs,
        CreateTranscriptionRequestArgs, CreateTranslationRequestArgs, CreateVectorStoreRequest,
        FilePurpose, FunctionObject, ImageDetail, ImageUrlArgs, MessageAttachment,
        MessageAttachmentTool, MessageContent, MessageContentTextAnnotations, MessageDeltaContent,
        MessageRole, ModifyAssistantRequest, RunObject, RunStatus, SpeechModel,
        SubmitToolOutputsRunRequest, TimestampGranularity, ToolsOutputs, Voice,
    },
    Client,
};
use futures::StreamExt;

#[tauri::command]
pub async fn assistants_vision_chat_test(
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
    println!("call assistents_stream_test: {:#?}", params);
    let postData = serde_json::from_str::<PostData>(params.as_str()).unwrap();
    match assistants_vision_chat_example(postData.message.unwrap_or_default().as_str())
        .await
        .map_err(|e| format!("{:?}", e))
    {
        Ok(_) => Ok("テスト終了".to_string()),
        Err(err) => {
            println!("Error: {:#?}", err);
            Err(err)
        }
    }
}

fn create_client() -> Client<OpenAIConfig> {
    Client::with_config(
        OpenAIConfig::new().with_api_key(API_KEY.read().map(|x| x.clone()).unwrap_or_default()),
    )
}

async fn assistants_vision_chat_example(question: &str) -> anyhow::Result<()> {
    let client = create_client();
    // let image_url = "https://upload.wikimedia.org/wikipedia/commons/thumb/d/dd/Gfp-wisconsin-madison-the-nature-boardwalk.jpg/2560px-Gfp-wisconsin-madison-the-nature-boardwalk.jpg";
    let image_url = "https://logos-world.net/wp-content/uploads/2020/09/Linux-Symbol.png";

    let request = CreateChatCompletionRequestArgs::default()
        .model("gpt-4o-mini")
        .max_tokens(300_u32)
        .messages([ChatCompletionRequestUserMessageArgs::default()
            .content(vec![
                ChatCompletionRequestMessageContentPartTextArgs::default()
                    .text("この絵は何ですか?")
                    .build()?
                    .into(),
                ChatCompletionRequestMessageContentPartImageArgs::default()
                    .image_url(
                        ImageUrlArgs::default()
                            .url(image_url)
                            .detail(ImageDetail::High)
                            .build()?,
                    )
                    .build()?
                    .into(),
            ])
            .build()?
            .into()])
        .build()?;

    println!("{}", serde_json::to_string(&request).unwrap());

    let response = client.chat().create(request).await?;

    println!("\nResponse:\n");
    for choice in response.choices {
        println!(
            "{}: Role: {}  Content: {:?}",
            choice.index,
            choice.message.role,
            choice.message.content.unwrap_or_default()
        );
    }

    Ok(())
}
