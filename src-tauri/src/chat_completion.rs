use crate::models::chat::ChatApiMessage;
use crate::util::{self, create_client};
use crate::{DIR_ASSISTANTS, SAVING_DIRECTORY};
use base64::prelude::*;
use futures::StreamExt;
use serde::Deserialize;
use serde_json::Value;
use std::fs::File;
use std::io::prelude::*;
use tauri::Window;

use async_openai::{
    config::OpenAIConfig,
    types::{
        self, AssistantStreamEvent, CreateAssistantRequestArgs, CreateFileRequestArgs,
        CreateImageRequestArgs, CreateMessageRequestArgs, CreateMessageRequestContent,
        CreateRunRequestArgs, CreateThreadRequestArgs, FileInput, ImageFile, ImageInput, ImageUrl,
        MessageContentInput, MessageDeltaContent, MessageRequestContentTextObject, MessageRole,
        RunObject, SubmitToolOutputsRunRequest, ToolsOutputs,
        ChatCompletionRequestMessageContentPartImageArgs, ChatCompletionRequestUserMessageContent,
        ChatCompletionRequestMessageContentPartTextArgs, ChatCompletionRequestMessage, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs, ImageDetail, ImageUrlArgs,ChatCompletionRequestMessageContentPart,
    },
    Client,
};



#[tauri::command]
pub async fn start_chat(
    window: Window,
    app_handle: tauri::AppHandle,
    params: String,
    timeout_sec: Option<u64>,
) -> Result<String, String> {
    println!("timeout {:?}", timeout_sec);

    #[derive(Deserialize, Debug)]
    struct PostData {
        model: Option<String>,
        messages: Vec<ChatApiMessage>,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
        imageUrl: Option<String>,
        filename: Option<String>,
        filebody: Option<String>,
        messageId: String,
    }
    let post_data = serde_json::from_str::<PostData>(params.as_str()).unwrap();
    // let messages: Vec<ChatApiMessage> = serde_json::from_str(message.as_str()).unwrap();
    println!("post_data:{:?}", &post_data);
    match exec_chat(
        window,
        post_data.model.unwrap_or("gpt-4o-mini".to_string()).as_str(),
        post_data.messages,
        post_data.temperature.unwrap_or(1.0),
        post_data.max_tokens.unwrap_or(1024),
        post_data.imageUrl,
        post_data.filename.unwrap_or_default().as_str(),
        post_data.filebody.unwrap_or_default().as_str(),
        post_data.messageId.as_str(),
    )
    .await
    .map_err(|e| format!("{:?}", e))
    {
        Ok(_) => Ok("テスト終了".to_string()),
        Err(err) => Err(err),
    }
}

async fn exec_chat(window: Window, model: &str, messages: Vec<ChatApiMessage>, temperature: f32, max_tokens: u32, image_url: Option<String>, file_name: &str, file_body: &str, message_id: &str) -> anyhow::Result<()> {
    //create a client
    let client = create_client()?;

    //create the assistant
    let content_list: Vec<String> = messages.iter().map(|x| x.content.clone()).collect();

    let mut image_file_id = "".to_string();
    let mut message_vec: Vec<ChatCompletionRequestMessageContentPart> = vec![];

    for content in content_list.iter() {
        if let Some(image_url) = image_url.clone().take_if(|x| !x.is_empty()) {
            // let image_url_build = types::ImageUrlArgs::default()
            //     .url(image_url.to_string())
            //     .build()?;
            // let image_url = MessageContentInput::ImageUrl(types::MessageContentImageUrlObject {
            //     image_url: image_url_build,
            // });
            // let image_url_message: ChatCompletionRequestMessage = ;
            message_vec.push(ChatCompletionRequestMessageContentPartImageArgs::default()
            .image_url(
                ImageUrlArgs::default()
                    .url(image_url.as_str())
                    .detail(ImageDetail::Auto)
                    .build()?,
            ).build()?.into());
            // let content_vec: Vec<MessageContentInput> = vec![image_url];
            // let image_url_message = CreateMessageRequestArgs::default()
            //     .role(MessageRole::User)
            //     .content(CreateMessageRequestContent::ContentArray(content_vec))
            //     .build()?;
            // let _message_obj_url = client
            //     .threads()
            //     .messages(&thread.id)
            //     .create(image_url_message)
            //     .await?;
        }
        if !file_name.is_empty() && !file_body.is_empty() && file_body.contains("base64,") {
            println!("img:{} ", file_body);
            // let file_binary: Vec<u8>;
            // //////////////////////////////////////////////////////////////////////////////////////////
            // if let Some((file_type, file_body)) = file_body.split_once("base64,") {
            //     file_binary = BASE64_STANDARD.decode(file_body)?;
            // } else {
            //     return Err(anyhow::anyhow!("Invalid file format"));
            // }
            // let base64_data_url = format!("data:image/jpeg;{}", file_body);

            
            message_vec.push(ChatCompletionRequestMessageContentPartImageArgs::default()
            .image_url(
                ImageUrlArgs::default()
                    .url(file_body)
                    .detail(ImageDetail::Auto)
                    .build()?,
            ).build()?.into());
        }

        message_vec.push(ChatCompletionRequestMessageContentPartTextArgs::default()
        .text(content.as_str())
        .build()?.into());
            // ChatCompletionRequestUserMessageArgs::default()
            // .content(message)
            // .build()?
            // .into()
    }
    
// let m = [ChatCompletionRequestUserMessageArgs::default()
//         .content("message")
//         .build()?
//         .into()];
    let request = CreateChatCompletionRequestArgs::default()
    .model(model)
    .max_tokens(max_tokens)
    .temperature(temperature)
    .stream(true)
    .messages([
        ChatCompletionRequestUserMessageArgs::default().content(message_vec).build()?.into()
    ])
    .build()?;

    let mut stream = client.chat().create_stream(request).await?;

    // let task =
    // tokio::spawn(async move {
        // let start_time = chrono::Utc::now();
        let mut prev_time = chrono::Utc::now();
        let mut response_string = String::new();
        let mut finish_with_error = false;
        while let Some(result) = stream.next().await {
            match result {
                Ok(response) => {
                    response.choices.iter().for_each(|chat_choice| {
                        if let Some(ref content) = chat_choice.delta.content {
                            // write!(lock, "{}", content).unwrap();
                            // let message = ChatApiMessage {
                            //     role: "assistant".to_string(),
                            //     content: content.to_string(),
                            // };
                            print!("{} ", content);
                            let now = chrono::Utc::now();
                            let duration = now - prev_time;
                            response_string.push_str(content.as_str());

                            if duration.gt(&chrono::Duration::milliseconds(200)) {
                                prev_time = now;
                                window.emit("stream_chunk", serde_json::json!({
                                "messageId": message_id.clone(), 
                                "response": response_string.clone(), 
                                "responseHtml": markdown::to_html(&response_string)
                            }))
                            .unwrap();
                            }

                        }
                    });
                }
                Err(err) => {
                    window.emit("stream_error", serde_json::json!({
                        "type": "OpenAIError",
                        "message": format!("{:?}", err)
                    })).unwrap();
                    finish_with_error = true;
                }
            }
        }
    
    // println!(
    //     "finish... markdown::to_html(&response_string):{:?}",
    //     markdown::to_html(&response_string)
    // );
    if !finish_with_error {
        window
        .emit("finish_chunks", serde_json::json!({
            "messageId": message_id.clone(), 
            "response": response_string.clone(), 
            // "responseHtml":  markdown::to_html(&response_string)
            "responseHtml":  markdown::to_html(&response_string)
        }))
        .unwrap();
    }
    
    // });
    Ok(())
}
