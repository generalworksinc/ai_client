use std::fs::File;
use std::io::prelude::*;
use std::str::FromStr;
use async_openai::types::AssistantObject;
use serde_json::Value;
use tauri::{CustomMenuItem, Menu, MenuItem, Submenu};
use tauri::{Manager, Window, WindowUrl};
use serde::Deserialize;
use crate::util::create_client;
use crate::{SAVING_DIRECTORY, DIR_ASSISTANTS };

use serde::{Serialize};
use chrono::{DateTime, Utc, TimeZone};
use once_cell::sync::{Lazy, OnceCell};

use async_openai::{
    types::{
        CreateAssistantRequestArgs, CreateMessageRequestArgs, CreateRunRequestArgs,
        CreateThreadRequestArgs, MessageContent, MessageRole, RunStatus,
    },
    Client,
    config::OpenAIConfig,
};
use crate::API_KEY;



#[tauri::command]
pub async fn reflesh_assistants(
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    let dir = unsafe{SAVING_DIRECTORY.clone()};
    let assistants_path = std::path::Path::new(dir.as_str()).join(DIR_ASSISTANTS);
    if assistants_path.exists() {
        if let Ok(read_dir) = assistants_path.read_dir() {
            let data_vec = read_dir.filter_map(|entry| {
                if let Ok(entry) = entry {
                    let datetime = Utc.timestamp_nanos(
                        entry.metadata().unwrap().modified().unwrap().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap().as_nanos() as i64
                    );
                    let data = std::fs::read_to_string(entry.path()).unwrap();
                    let json_data = serde_json::from_str::<Value>(data.as_str()).unwrap();
                    // json_data.insert("time", datetime.format("%Y-%m-%d %H:%M:%S").to_string());
                    Some(json_data)
                } else {
                    None
                }
            }).collect::<Vec<Value>>();
            return Ok(serde_json::to_string(&data_vec).unwrap());
        }
    }
    return Err("".to_string());
}

#[tauri::command]
pub async fn make_assistant(window: Window,
    app_handle: tauri::AppHandle,
    params: String,
    timeout_sec: Option<u64>,) -> Result<String, String> {
    #[derive(Deserialize)]
    struct PostData {
        message: Option<String>,
        assistant_name: String,
        instructions: Option<String>,
    }
    println!("call assistents_test: {:#?}", params);
    let postData = serde_json::from_str::<PostData>(params.as_str()).unwrap();
    match exec(&postData.assistant_name, postData.message.unwrap_or_default().as_str(), postData.instructions.unwrap_or_default().as_str()).await.map_err(|e| format!("{:?}", e)) {
        Ok(_) => {
            Ok("テスト終了".to_string())
        }
        Err(err) => {
            Err(err)
        }
    }
}

async fn exec(assistant_name: &str, question: &str, instructions: &str) -> anyhow::Result<()>{
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
    let dir = unsafe{SAVING_DIRECTORY.clone()};
    
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
    f.write_all(json_data.as_bytes());

    client.assistants().delete(&assistant_object.id).await?;
    Ok(())
}