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
pub async fn assistants_tool_calls_test(awindow: Window,
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
    match assistants_tool_calls_example(postData.message.unwrap_or_default().as_str()).await.map_err(|e| format!("{:?}", e)) {
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

async fn assistants_tool_calls_example(question: &str) -> anyhow::Result<()>{
    let client = create_client();

    let whether_function =  FunctionObjectArgs::default()
    .name("get_current_weather")
    .description("特定の場所の現在の天気を取得する")
    .parameters(json!({
        "type": "object",
        "properties": {
            "location": {
                "type": "string",
                "description": "The city and state, e.g. San Francisco, CA",
            },
            "unit": { "type": "string", "enum": ["celsius", "fahrenheit"] },
        },
        "required": ["location"],
    }))
    .build()?;

    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(512u32)
        .model("gpt-4o-mini")
        .messages([ChatCompletionRequestUserMessageArgs::default()
            .content("ボストンとアトランタの気候は?")
            .build()?
            .into()])
        .tools(vec![ChatCompletionToolArgs::default()
            .r#type(ChatCompletionToolType::Function)
            .function(whether_function)
            .build()?])
        .build()?;

    let response_message = client
        .chat()
        .create(request)
        .await?
        .choices
        .first()
        .unwrap()
        .message
        .clone();

    if let Some(tool_calls) = response_message.tool_calls {
        let mut handles = Vec::new();
        for tool_call in tool_calls {
            let name = tool_call.function.name.clone();
            let args = tool_call.function.arguments.clone();
            let tool_call_clone = tool_call.clone();

            let handle =
                tokio::spawn(async move { call_fn(&name, &args).await.unwrap_or_default() });
            handles.push((handle, tool_call_clone));
        }

        let mut function_responses = Vec::new();

        for (handle, tool_call_clone) in handles {
            if let Ok(response_content) = handle.await {
                function_responses.push((tool_call_clone, response_content));
            }
        }

        let mut messages: Vec<ChatCompletionRequestMessage> =
            vec![ChatCompletionRequestUserMessageArgs::default()
                .content("ボストンとアトランタの気候は?")
                .build()?
                .into()];

        let tool_calls: Vec<ChatCompletionMessageToolCall> = function_responses
            .iter()
            .map(|(tool_call, _response_content)| tool_call.clone())
            .collect();

        let assistant_messages: ChatCompletionRequestMessage =
            ChatCompletionRequestAssistantMessageArgs::default()
                .tool_calls(tool_calls)
                .build()?
                .into();

        let tool_messages: Vec<ChatCompletionRequestMessage> = function_responses
            .iter()
            .map(|(tool_call, response_content)| {
                ChatCompletionRequestToolMessageArgs::default()
                    .content(response_content.to_string())
                    .tool_call_id(tool_call.id.clone())
                    .build()
                    .unwrap()
                    .into()
            })
            .collect();

        messages.push(assistant_messages);
        messages.extend(tool_messages);

        let subsequent_request = CreateChatCompletionRequestArgs::default()
            .max_tokens(512u32)
            .model("gpt-4o-mini")
            .messages(messages)
            .build()?;

        let mut stream = client.chat().create_stream(subsequent_request).await?;

        let mut response_content = String::new();
        while let Some(result) = stream.next().await {
            match result {
                Ok(response) => {
                    for chat_choice in response.choices.iter() {
                        if let Some(ref content) = chat_choice.delta.content {
                            print!("{}", content);
                            response_content.push_str(content);
                        }
                    }
                }
                Err(err) => {
                    anyhow::bail!(err);
                }
            }
        }
    }
    Ok(())
}


async fn call_fn(name: &str, args: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let mut available_functions: HashMap<&str, fn(&str, &str) -> serde_json::Value> =
        HashMap::new();
    available_functions.insert("get_current_weather", get_current_weather);

    let function_args: serde_json::Value = args.parse().unwrap();

    let location = function_args["location"].as_str().unwrap();
    let unit = function_args["unit"].as_str().unwrap_or("fahrenheit");
    let function = available_functions.get(name).unwrap();
    let function_response = function(location, unit);
    Ok(function_response)
}

fn get_current_weather(location: &str, unit: &str) -> serde_json::Value {
    let mut rng = thread_rng();

    let temperature: i32 = rng.gen_range(20..=55);

    let forecasts = [
        "sunny", "cloudy", "overcast", "rainy", "windy", "foggy", "snowy",
    ];

    let forecast = forecasts.choose(&mut rng).unwrap_or(&"sunny");

    let weather_info = json!({
        "location": location,
        "temperature": temperature.to_string(),
        "unit": unit,
        "forecast": forecast
    });

    weather_info
}