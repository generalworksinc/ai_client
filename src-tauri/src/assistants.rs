use crate::models::chat::ChatApiMessage;
use crate::util::create_client;
use crate::{DIR_ASSISTANTS, SAVING_DIRECTORY};
use futures::StreamExt;
use serde::Deserialize;
use serde_json::Value;
use std::fs::File;
use std::io::prelude::*;
use tauri::Window;

use async_openai::{
    config::OpenAIConfig,
    types::{
        AssistantStreamEvent, CreateAssistantRequestArgs, CreateMessageRequestArgs,
        CreateRunRequestArgs, CreateThreadRequestArgs, MessageDeltaContent, MessageRole, RunObject,
        SubmitToolOutputsRunRequest, ToolsOutputs,
    },
    Client,
};

#[tauri::command]
pub async fn delete_assistant(app_handle: tauri::AppHandle, id: String) -> Result<String, String> {
    //create a client
    let client = create_client().map_err(|x| format!("failed to create client: {:?}", x))?;
    //delete assistant
    let delete_err = client.assistants().delete(id.as_str()).await.err();
    if let Some(error) = delete_err {
        let delete_err_str = format!("failed to delete assistant: {:?}", error);
        //すでに削除されている場合はエラーを無視
        if !delete_err_str.contains("No assistant found") {
            return Err(delete_err_str);
        }
    }

    println!("delete assistant: {}", id);

    let dir = unsafe { SAVING_DIRECTORY.clone() };
    let file_path_assistants = std::path::Path::new(dir.as_str())
        .join(DIR_ASSISTANTS)
        .join(id.clone());

    if file_path_assistants.exists() {
        //削除
        std::fs::remove_file(file_path_assistants).map_err(|x| x.to_string())?;
    }

    Ok("削除しました".to_string())
}

#[tauri::command]
pub async fn reflesh_assistants(app_handle: tauri::AppHandle) -> Result<String, String> {
    let dir = unsafe { SAVING_DIRECTORY.clone() };
    let assistants_path = std::path::Path::new(dir.as_str()).join(DIR_ASSISTANTS);
    if assistants_path.exists() {
        if let Ok(read_dir) = assistants_path.read_dir() {
            let data_vec = read_dir
                .filter_map(|entry| {
                    if let Ok(entry) = entry {
                        // let datetime = Utc.timestamp_nanos(
                        //     entry
                        //         .metadata()
                        //         .unwrap()
                        //         .modified()
                        //         .unwrap()
                        //         .duration_since(std::time::SystemTime::UNIX_EPOCH)
                        //         .unwrap()
                        //         .as_nanos() as i64,
                        // );
                        let data = std::fs::read_to_string(entry.path()).unwrap();
                        let json_data = serde_json::from_str::<Value>(data.as_str()).unwrap();
                        // json_data.insert("time", datetime.format("%Y-%m-%d %H:%M:%S").to_string());
                        Some(json_data)
                    } else {
                        None
                    }
                })
                .collect::<Vec<Value>>();
            return Ok(serde_json::to_string(&data_vec).unwrap());
        }
    }
    Err("".to_string())
}

#[tauri::command]
pub async fn make_assistant(
    window: Window,
    app_handle: tauri::AppHandle,
    params: String,
    timeout_sec: Option<u64>,
) -> Result<String, String> {
    #[derive(Deserialize)]
    struct PostData {
        // message: Option<String>,
        assistant_name: String,
        instructions: Option<String>,
    }
    println!("call assistents_test: {:#?}", params);
    let postData = serde_json::from_str::<PostData>(params.as_str()).unwrap();
    match exec_make_assistant(
        &postData.assistant_name,
        // postData.message.unwrap_or_default().as_str(),
        postData.instructions.unwrap_or_default().as_str(),
    )
    .await
    .map_err(|e| format!("{:?}", e))
    {
        Ok(_) => Ok("テスト終了".to_string()),
        Err(err) => Err(err),
    }
}

async fn exec_make_assistant(assistant_name: &str, instructions: &str) -> anyhow::Result<()> {
    //create a client
    let client = create_client()?;

    //create the assistant
    let assistant_request = CreateAssistantRequestArgs::default()
        .name(assistant_name)
        .instructions(instructions)
        .model("gpt-4o-mini")
        .build()?;
    let assistant_object = client.assistants().create(assistant_request).await?;

    // client.assistants().delete(assistant_id).await?;

    //データをローカルに保存する
    let dir = unsafe { SAVING_DIRECTORY.clone() };

    // if file_path_conversation.exists() {
    //     //削除
    //     std::fs::remove_file(file_path_conversation).map_err(|x| x.to_string())?;
    // }
    // if file_path_title.exists() {
    //     //削除
    //     std::fs::remove_file(file_path_title).map_err(|x| x.to_string())?;
    // }

    // write_id and conversasion.
    let assistants_dir_path = std::path::Path::new(dir.as_str()).join(DIR_ASSISTANTS);

    if !assistants_dir_path.exists() {
        std::fs::create_dir_all(assistants_dir_path.as_path())?;
    }
    let file_path = assistants_dir_path.join(assistant_object.id.clone());
    let mut f = File::create(file_path).unwrap();
    let json_data = serde_json::to_string(&assistant_object)?;
    f.write_all(json_data.as_bytes())?;

    // client.assistants().delete(&assistant_object.id).await?;
    Ok(())
}

#[tauri::command]
pub async fn make_new_thread(
    window: Window,
    app_handle: tauri::AppHandle,
    params: String,
    timeout_sec: Option<u64>,
) -> Result<String, String> {
    println!("call make_new_thread: {:#?}", params);
    #[derive(Deserialize)]
    struct PostData {
        messages: Vec<ChatApiMessage>,
        assistant_id: String,
        // instructions: Option<String>,
        messageId: String,
        threadId: String,
    }
    println!("call assistents_test: {:#?}", params);
    let postData = serde_json::from_str::<PostData>(params.as_str()).unwrap();
    let message_id = postData.messageId.clone();

    match exec_make_new_thread(
        // &postData.assistant_name,
        window,
        postData.messages,
        postData.assistant_id.as_str(),
        message_id.as_str(),
        postData.threadId.as_str(), // postData.instructions.unwrap_or_default().as_str(),
    )
    .await
    .map_err(|e| format!("{:?}", e))
    {
        Ok(_) => Ok("テスト終了".to_string()),
        Err(err) => {
            eprintln!("{:?}", err);
            Err(err)
        }
    }
}

async fn exec_make_new_thread(
    window: Window,
    messages: Vec<ChatApiMessage>,
    assistant_id: &str,
    message_id: &str,
    thread_id: &str,
) -> anyhow::Result<()> {
    //create a client
    let client = create_client()?;

    // let query = [("limit", "1")]; //limit the list responses to 1 message
    let assistant = client.assistants().retrieve(assistant_id).await?;
    //get the id of the assistant
    let assistant_id = &assistant.id;
    println!("assistant_id: {:?}", assistant);
    // // Step 2: Create a Thread and add Messages

    //create a thread for the conversation
    let mut new_thread = false;
    let thread = if thread_id.is_empty() {
        new_thread = true;
        let thread_request = CreateThreadRequestArgs::default().build()?;
        client.threads().create(thread_request.clone()).await?
    } else {
        //threadが取得できなかったら、新規作成する
        match client.threads().retrieve(thread_id).await {
            Ok(thread) => thread,
            Err(e) => {
                let thread_request = CreateThreadRequestArgs::default().build()?;
                new_thread = true;
                client.threads().create(thread_request.clone()).await?
            }
        }
    };

    //create a message for the thread
    let content_list = if new_thread {
        messages.iter().map(|x| x.content.clone()).collect()
    } else {
        vec![messages.last().unwrap().content.clone()]
    };
    for content in content_list.iter() {
        let message = CreateMessageRequestArgs::default()
            .role(MessageRole::User)
            .content(content.as_str())
            .build()?;
        //attach message to the thread
        let _message_obj = client
            .threads()
            .messages(&thread.id)
            .create(message)
            .await?;
    }
    // let message = CreateMessageRequestArgs::default()
    //     .role(MessageRole::User)
    //     .content(message)
    //     .build()?;

    // //attach message to the thread
    // let _message_obj = client
    //     .threads()
    //     .messages(&thread.id)
    //     .create(message)
    //     .await?;

    println!("thread id: {:?}", thread.id);

    // Step 3: Initiate a Run
    //create a run for the thread
    let run_request = CreateRunRequestArgs::default()
        .assistant_id(assistant_id)
        .stream(true)
        .build()?;

    let mut event_stream = client
        .threads()
        .runs(&thread.id)
        .create_stream(run_request.clone())
        .await?;

    let mut task_handle = None;

    let mut prev_time = chrono::Utc::now();
    let mut response_string = String::new();

    while let Some(event) = event_stream.next().await {
        match event {
            Ok(event) => match event {
                AssistantStreamEvent::ThreadRunRequiresAction(run_object) => {
                    println!("thread.run.requires_action: run_id:{}", run_object.id);
                    let client = client.clone();
                    task_handle = Some(tokio::spawn(async move {
                        handle_requires_action(client, run_object).await
                    }));
                }
                AssistantStreamEvent::ThreadMessageDelta(delta) => {
                    if let Some(contents) = delta.delta.content {
                        for content in contents {
                            // only text is expected here and no images
                            match content {
                                MessageDeltaContent::Text(text) => {
                                    if let Some(text) = text.text {
                                        if let Some(text) = text.value {
                                            print!("{} ", text);
                                            let now = chrono::Utc::now();
                                            let duration = now - prev_time;
                                            response_string.push_str(&text);
                                            if duration.gt(&chrono::Duration::milliseconds(200)) {
                                                prev_time = now;
                                                window.emit("stream_chunk", serde_json::json!({
                                                    "messageId": message_id, 
                                                    "threadId": thread.id,
                                                    "response": response_string.clone(), 
                                                    "responseHtml": markdown::to_html(&response_string)
                                                }))
                                                .unwrap();
                                            }
                                        }
                                    }
                                }
                                MessageDeltaContent::ImageFile(image) => {
                                    if let Some(image) = image.image_file {
                                        println!("Image file id: {:?}", image.file_id);
                                    }
                                }
                                MessageDeltaContent::ImageUrl(imageUrl) => {
                                    if let Some(image_url) = imageUrl.image_url {
                                        println!("image_url: {:?}", image_url);
                                    }
                                }
                            }
                        }
                    }
                }
                AssistantStreamEvent::ThreadMessageCompleted(messageCompleted) => {
                    println!(
                        "messageCompleted. RunId {:?}: {:?}",
                        messageCompleted.content, messageCompleted.attachments
                    );
                    // match content {
                    //     MessageDeltaContent::Text(text) => {
                    //         if let Some(text) = text.text {
                    //             if let Some(text) = text.value {
                    //                 print!("{} ", text);
                    //                 window.emit("stream_chunk", serde_json::json!({
                    //                     "messageId": messageId.clone(),
                    //                     "response": response_string.clone(),
                    //                     "responseHtml": markdown::to_html(&response_string)
                    //                 }))
                    //                 .unwrap();
                    //             }
                    //         }
                    //     }
                    //     MessageDeltaContent::ImageFile(image) => {
                    //         if let Some(image) = image.image_file {
                    //             println!("Image file id: {:?}", image.file_id);
                    //         }
                    //     }
                    //     MessageDeltaContent::ImageUrl(imageUrl) => {
                    //         if let Some(image_url) = imageUrl.image_url {
                    //             println!("image_url: {:?}", image_url);
                    //         }
                    //     }
                    // }
                    window
                        .emit(
                            "finish_chunks",
                            serde_json::json!({
                                "messageId": message_id,
                                "response": response_string.clone(),
                                "responseHtml": markdown::to_html(&response_string),
                                "threadId": thread.id,
                            }),
                        )
                        .unwrap();
                }
                AssistantStreamEvent::ThreadRunStepCompleted(runStepComplete) => {
                    println!(
                        "StepCompleted. RunId {:?}: {:?}",
                        runStepComplete.id, runStepComplete.completed_at
                    );
                }
                AssistantStreamEvent::ThreadRunCompleted(run) => {
                    println!("RunCompleted. RunId {:?}: {:?}", run.id, run.completed_at);
                }
                // AssistantStreamEvent::ThreadRunStepExpired(delta) => {
                //     eprintln!("StepExpired. RunId {:?}: {:?}", delta.id, delta.expired_at);
                //     window.emit("timeout_stream", message_id).unwrap();
                // }
                AssistantStreamEvent::ThreadMessageIncomplete(delta) => {
                    window
                        .emit(
                            "finish_chunks",
                            serde_json::json!({
                                "messageId": message_id,
                                "response": response_string.clone(),
                                "responseHtml": markdown::to_html(&response_string),
                                "threadId": thread.id,
                            }),
                        )
                        .unwrap();
                }
                AssistantStreamEvent::Done(delta) => {
                    println!("Done. {:?}", delta);
                }
                _ => {
                    // println!("\nEvent: {event:?}\n")
                }
            },
            Err(e) => {
                eprintln!("Error: {e}");
                window
                    .emit("stream_openai_error", format!("{:?}", e))
                    .unwrap();

                // window.emit("timeout_stream", message_id).unwrap();
                // break;
            }
        }
    }

    // wait for task to handle required action and submit tool outputs
    if let Some(task_handle) = task_handle {
        let _ = tokio::join!(task_handle);
    }

    // //once we have broken from the main loop we can delete the assistant and thread
    // println!("assistant_id: {:?}", assistant_id);
    // println!("thread_id: {:?}", thread.id);
    // client.assistants().delete(assistant_id).await?;
    // client.threads().delete(&thread.id).await?;
    client
        .threads()
        .delete("thread_bLoskYkBNlDDlO8Ibm4TncLl")
        .await;
    // client
    //     .threads()
    //     .delete("thread_w1u1EKk34jDqXJBeJj6wQ2Ye")
    //     .await?;
    println!("thread_id: {:?}", thread.id);
    Ok(())
}

async fn handle_requires_action(client: Client<OpenAIConfig>, run_object: RunObject) {
    let mut tool_outputs: Vec<ToolsOutputs> = vec![];
    if let Some(ref required_action) = run_object.required_action {
        for tool in &required_action.submit_tool_outputs.tool_calls {
            if tool.function.name == "get_current_temperature" {
                tool_outputs.push(ToolsOutputs {
                    tool_call_id: Some(tool.id.clone()),
                    output: Some("57".into()),
                })
            }

            if tool.function.name == "get_rain_probability" {
                tool_outputs.push(ToolsOutputs {
                    tool_call_id: Some(tool.id.clone()),
                    output: Some("0.06".into()),
                })
            }
        }

        if let Err(e) = submit_tool_outputs(client, run_object, tool_outputs).await {
            eprintln!("Error on submitting tool outputs: {e}");
        }
    }
}

async fn submit_tool_outputs(
    client: Client<OpenAIConfig>,
    run_object: RunObject,
    tool_outputs: Vec<ToolsOutputs>,
) -> anyhow::Result<()> {
    let mut event_stream = client
        .threads()
        .runs(&run_object.thread_id)
        .submit_tool_outputs_stream(
            &run_object.id,
            SubmitToolOutputsRunRequest {
                tool_outputs,
                stream: Some(true),
            },
        )
        .await?;

    while let Some(event) = event_stream.next().await {
        match event {
            Ok(event) => {
                if let AssistantStreamEvent::ThreadMessageDelta(delta) = event {
                    if let Some(contents) = delta.delta.content {
                        for content in contents {
                            // only text is expected here and no images
                            if let MessageDeltaContent::Text(text) = content {
                                if let Some(text) = text.text {
                                    if let Some(text) = text.value {
                                        print!("{}", text);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Error: {e}");
            }
        }
    }

    Ok(())
}
