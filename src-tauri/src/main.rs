// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(warnings)]

mod assistants;
mod assistants_audio_speech;
mod assistants_audio_transcribe;
mod assistants_audio_translate;
mod assistants_code_interpreter;
mod assistants_example;
mod assistants_file_search;
mod assistants_stream;
mod assistants_tool_calls;
mod assistants_vision_chat;
mod audio;
mod chat_completion;
mod constants;
mod embedding;
mod models;
mod open_ai_files;
mod util;
mod infrastracture;

use constants::{DIR_CONVERSATION, DIR_THREADS};
use futures::future;
use futures::stream::StreamExt;
use models::chat::ChatApiMessage;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, Read, Write};
use std::path::PathBuf;
use std::time::Duration;
use tauri::{Emitter, Window};
// use tauri::{CustomMenuItem, Menu, Submenu};
use tauri::menu::{MenuBuilder, MenuItemBuilder, Submenu, SubmenuBuilder, PredefinedMenuItem, CheckMenuItemBuilder, MenuId};
use tauri::{path::BaseDirectory, Manager};

// use futures_util::stream::StreamExt;
use rand::prelude::*;
use util::create_client;

use chrono::{TimeZone, Utc};
use once_cell::sync::OnceCell;
use reqwest::header;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

use std::sync::RwLock;
// pub static mut API_KEY: String = String::new();
pub static API_KEY: LazyLock<RwLock<String>> = LazyLock::new(|| RwLock::new(String::new()));

pub static mut SAVING_DIRECTORY: String = String::new();
pub static PATH_DIR_CHATGPT_CONFIG: OnceCell<PathBuf> = OnceCell::new();

pub fn create_reqwest_client() -> reqwest::Client {
    // certificate使ってサーバからデータ取得する
    let mut headers = header::HeaderMap::new();
    headers.insert(
        "Content-Type",
        header::HeaderValue::from_static("application/json"),
    );
    headers.insert("X-Slack-No-Retry", header::HeaderValue::from(1));

    unsafe {
        headers.insert(
            "Authorization",
            header::HeaderValue::from_str(
                format!("Bearer {}", API_KEY.read().unwrap().as_str()).as_str(),
            )
            .unwrap(),
        );
    }

    reqwest::Client::builder()
        .use_rustls_tls()
        .default_headers(headers)
        .build()
        .unwrap()
}

////////ChatGPT API Response /////////////////////////////////////////////////////

#[derive(Serialize, Deserialize, Debug, Default)]
struct AIClientConversation {
    pub assistant_id: Option<String>,
    pub title: String,
    pub messages: Vec<ChatApiMessageWithHtml>,
    // pub name: Option<String>,
    pub id: String,
    pub time: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ChatApiMessageWithHtml {
    role: String,
    content: String,
    content_html: Option<String>,
}
#[derive(Serialize, Deserialize, Debug)]
struct ChatApiSendMessage {
    model: String,
    max_tokens: u32,
    temperature: f32,
    messages: Vec<ChatApiMessage>,
    stream: bool,
}

#[derive(Serialize, Deserialize)]
struct ChatGptResponseData {
    error: Option<ChatGptError>,
    id: Option<String>,
    object: Option<String>,
    created: Option<i64>,
    model: Option<String>,
    choices: Option<Vec<Choice>>,
}
#[derive(Serialize, Deserialize)]
struct ChatGptError {
    message: Option<String>,
    #[serde(rename = "type")]
    error_type: Option<String>,
}
#[derive(Serialize, Deserialize)]
struct ChatGptStopResponseData {
    error: Option<ChatGptError>,
    id: Option<String>,
    object: Option<String>,
    created: Option<i64>,
    model: Option<String>,
    choices: Option<Vec<ChoiceStop>>,
}
#[derive(Serialize, Deserialize)]
struct ChoiceStop {
    finish_reason: Option<String>,
    index: i32,
    message: ChatApiMessage,
}
#[derive(Serialize, Deserialize)]
struct Choice {
    delta: Delta,
    index: i32,
    finish_reason: Option<String>,
}
#[derive(Serialize, Deserialize)]
struct Delta {
    role: Option<String>,
    content: Option<String>,
}

////////Config toml file /////////////////////////////////////////////////////
#[derive(Serialize, Deserialize, Default)]
struct Config {
    //    ip: String,
    //    port: Option<u16>,
    pub keys: Keys,
    pub saving_directory: Option<String>,
}

#[derive(Serialize, Deserialize, Default)]
struct Keys {
    pub chatgpt: Option<String>,
    //    travis: Option<String>,
}

#[derive(Deserialize, Serialize, Default)]
struct ThreadData {
    pub id: Option<String>,
    pub name: Option<String>,
    pub object: Option<String>,
    pub created_at: Option<i64>,
    pub time: Option<String>,
    // {"id":"thread_aFRZqocRwwAJQ0wTBphELg1v","object":"thread","created_at":1723108843,"tool_resources":{},"metadata":{}}
}

#[derive(Deserialize, Serialize)]
struct TitleData {
    pub id: String,
    pub name: String,
    pub time: String,
}

#[tauri::command]
async fn set_api_key(
    app_handle: tauri::AppHandle,
    api_key: String,
    saving_directory: String,
) -> Result<String, String> {
    let chat_gpt_config_dir = PATH_DIR_CHATGPT_CONFIG.get().unwrap();

    let config_toml_file_path = chat_gpt_config_dir.join("config.toml");

    if !config_toml_file_path.exists() {
        std::fs::create_dir_all(chat_gpt_config_dir.clone()).unwrap();
        let mut f = File::create(config_toml_file_path.clone()).unwrap();
        let config_file = toml::to_string(&Config::default()).unwrap();
        f.write_all(config_file.as_bytes())
            .expect("Unable to write config data");
    }

    if let Ok(f) = File::open(config_toml_file_path.clone()) {
        let mut buf_reader = BufReader::new(f);
        let mut config_data: Vec<u8> = vec![];
        buf_reader.read_to_end(&mut config_data).unwrap();
        let mut config: Config =
            toml::from_str(String::from_utf8(config_data).unwrap().as_str()).unwrap();

        config.keys.chatgpt = Some(api_key.clone());
        let saving_dir_opt = if saving_directory.is_empty() {
            None
        } else {
            unsafe {
                SAVING_DIRECTORY = saving_directory.clone();
            }
            Some(saving_directory.clone())
        };
        config.saving_directory = saving_dir_opt;
        let config_file = toml::to_string(&config).unwrap();
        let mut f = File::create(config_toml_file_path.clone()).unwrap();
        f.write_all(config_file.as_bytes())
            .expect("Unable to write config data");
        // unsafe {
        //     API_KEY = api_key;
        // }
        {
            let mut key = API_KEY.write().unwrap();
            *key = api_key;
        }
        return Ok("save config data.".into());
    }

    Ok("can't save config data.".into())
}
#[tauri::command]
async fn get_api_key(app_handle: tauri::AppHandle) -> Result<String, String> {
    let a = API_KEY.read().unwrap().clone();
    let res = unsafe {
        serde_json::json!({
            "apiKey": a,
            "savingDirectory": SAVING_DIRECTORY.clone(),
        })
    };
    Ok(res.to_string())
}

#[tauri::command]
async fn change_message(
    app_handle: tauri::AppHandle,
    id: String,
    title: String,
) -> Result<String, String> {
    let dir = unsafe { SAVING_DIRECTORY.clone() };
    let file_path_conversation = std::path::Path::new(dir.as_str())
        .join(DIR_CONVERSATION)
        .join(id.clone());

    if file_path_conversation.exists() {
        let mut conversation_obj: AIClientConversation = serde_json::from_str(
            std::fs::read_to_string(file_path_conversation.as_path())
                .unwrap()
                .as_str(),
        )
        .unwrap();
        conversation_obj.title = title;
        std::fs::write(
            file_path_conversation,
            serde_json::to_string(&conversation_obj).unwrap(),
        )
        .map_err(|err| err.to_string())?;
        //更新
        // title_f.write_all(title.as_bytes()).unwrap();
    }
    Ok("変更しました".to_string())
}

#[tauri::command]
async fn delete_thread(app_handle: tauri::AppHandle, id: String) -> Result<String, String> {
    //Threadを削除
    println!("thread_id: {:#?}", id);
    let client = create_client().map_err(|err| err.to_string())?;
    match client
        .threads()
        .delete(id.as_str())
        .await
        .map_err(|x| x.to_string())
    {
        Ok(_) => {
            println!("thread delete success {:?}", id);
        }
        Err(err) => {
            eprintln!("thread delete error: {:#?}", err);
        }
    }

    let dir = unsafe { SAVING_DIRECTORY.clone() };

    //threadファイルを削除
    let file_path_thread = std::path::Path::new(dir.as_str())
        .join(DIR_THREADS)
        .join(id.clone());
    if file_path_thread.exists() {
        //削除
        std::fs::remove_file(file_path_thread).map_err(|x| x.to_string())?;
    }

    //conversationがthreadひも付きになっていた場合、ひも付きがなくなるので、ファイル名を変更する
    let file_path_conversation = std::path::Path::new(dir.as_str())
        .join(DIR_CONVERSATION)
        .join(id.clone());
    if file_path_conversation.exists() {
        //ID付け替える
        let new_id = uuid::Uuid::new_v4().to_string();
        let new_path = std::path::Path::new(dir.as_str())
            .join(DIR_CONVERSATION)
            .join(new_id.as_str());
        std::fs::rename(file_path_conversation.as_path(), new_path.as_path())
            .map_err(|x| x.to_string())?;
        let file_data_str =
            std::fs::read_to_string(new_path.as_path()).map_err(|x| x.to_string())?;
        let mut conversation: AIClientConversation =
            serde_json::from_str(file_data_str.as_str()).map_err(|x| x.to_string())?;
        conversation.id = new_id;
        let file_data_updated_str = serde_json::to_string(&conversation).unwrap();
        std::fs::write(new_path.as_path(), file_data_updated_str).map_err(|x| x.to_string())?;
    }

    Ok("ID変更しました".to_string())
}

#[tauri::command]
async fn delete_message(app_handle: tauri::AppHandle, id: String) -> Result<String, String> {
    let dir = unsafe { SAVING_DIRECTORY.clone() };
    let file_path_conversation = std::path::Path::new(dir.as_str())
        .join(DIR_CONVERSATION)
        .join(id.clone());

    if file_path_conversation.exists() {
        //削除
        std::fs::remove_file(file_path_conversation).map_err(|x| x.to_string())?;
    }

    //Threadの場合、削除します

    if !id.is_empty() && util::is_thread(id.as_str()) {
        println!("thread_id: {:#?}", id);
        delete_thread(app_handle, id.clone()).await?;
    }

    Ok("削除しました".to_string())
}

#[tauri::command]
async fn save_chat(app_handle: tauri::AppHandle, params: String) -> Result<String, String> {
    #[derive(Deserialize, Serialize)]
    struct PostData {
        data: Vec<ChatApiMessage>,
        // data: String,
        id: Option<String>,
        thread_id: Option<String>,
        assistant_id: Option<String>,
        save_thread: Option<bool>,
    }
    println!("params: {:#?}", params);
    let post_data = serde_json::from_str::<PostData>(params.as_str()).unwrap();

    //make file title
    let mut title_content = "".to_string();
    let messages = post_data.data;
    for message in &messages {
        println!("message: {:#?}", message);
        if message.role == "user" {
            title_content += message.content.as_str();
            break;
        } else if message.role == "assistant" {
            title_content += message.content.as_str();
            break;
        }
    }
    if title_content.len() > 30 {
        match get_title(title_content.clone()).await {
            Ok(title) => {
                title_content = title;
            }
            Err(err) => {
                println!("err: {:#?}", err);
                // title_f.write_all(title_content.as_bytes()).unwrap();
                return Err("".to_string());
            }
        }
    }

    //threadの保存が不要な場合、削除する
    let thread_id = if let Some(thread_id) = post_data.thread_id.filter(|x| !x.is_empty()) {
        if post_data.save_thread == Some(true) {
            thread_id
        } else {
            delete_thread(app_handle, thread_id.clone()).await?;
            "".to_string()
        }
    } else {
        "".to_string()
    };

    // write_message into file.
    let id = if !thread_id.is_empty() {
        thread_id
    } else if let Some(id) = post_data.id.clone() {
        id
    } else {
        uuid::Uuid::new_v4().to_string()
    };

    // write_id and conversasion.
    let dir = unsafe { SAVING_DIRECTORY.clone() };
    let content_dir_path = std::path::Path::new(dir.as_str()).join(DIR_CONVERSATION);

    if !content_dir_path.exists() {
        if let Err(_) = std::fs::create_dir_all(content_dir_path.as_path()) {
            return Err("can't create title directory.".into());
        }
    }
    let file_path = content_dir_path.join(id.clone());
    let mut f = File::create(file_path).unwrap();
    // let messages = serde_json::from_str::<Vec<ChatApiMessageWithHtml>>(post_data.data)?;
    let ai_client_conversation_data = AIClientConversation {
        id: id.clone(),
        assistant_id: post_data.assistant_id.clone(),
        title: title_content,
        messages: messages
            .into_iter()
            .map(|x| x.convert_with_html())
            .collect(),
        ..Default::default()
    };
    f.write_all(
        serde_json::to_string(&ai_client_conversation_data)
            .unwrap()
            .as_bytes(),
    )
    .unwrap();

    refresh_index_db().unwrap();

    Ok("".to_string())
}

#[tauri::command]
async fn reflesh_index(app_handle: tauri::AppHandle) -> Result<String, String> {
    refresh_index_db().unwrap();
    Ok("".to_string())
}
fn from_u8_to_str(buf: &[u8]) -> &str {
    let s = match std::str::from_utf8(buf) {
        Ok(v) => v,
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };
    s
}
#[tauri::command]
async fn search_conversations(
    app_handle: tauri::AppHandle,
    word: String,
) -> Result<String, String> {
    //search sled database
    let tree = sled::open(PATH_DIR_CHATGPT_CONFIG.get().unwrap().join("storage")).unwrap();
    let title_tree = tree.open_tree("title").unwrap();
    println!("word: {:#?}", word);
    for entryResult in tree.iter() {
        if let Ok((key, value)) = entryResult {
            let key = from_u8_to_str(&key);
            let id = from_u8_to_str(&value);
            // println!("id: {:#?}", id);
            // println!("body: {:#?}", key);
        }
    }

    // Iterates over key-value pairs, starting at the given key.

    // let mut iter = tree.range(word.as_bytes()..);
    let response = tree
        .iter()
        .flatten()
        .filter_map(|(key, value)| {
            // println!("--------------------------------------------------------------------------------------------------------");
            let body: String = std::str::from_utf8(&key).unwrap_or_default().to_string();
            let is_contains = body.contains(word.as_str());
            // let contains_binary = key.binary_search_by_key(|x| word.as_bytes());
            // println!("is_contains: {:#?}", is_contains);
            // println!("contains_binary: {:#?}", contains_binary);
            if !is_contains {
                return None;
            }
            let id: String = std::str::from_utf8(&value).unwrap_or_default().to_string();
            let title = if let Ok(Some(title)) = title_tree.get(id.as_str()) {
                std::str::from_utf8(&title).unwrap_or_default().to_string()
            } else {
                "".to_string()
            };
            println!("id: {:#?}", id);
            Some(serde_json::json!({
                "id": id,
                "title": title,
                "body": std::str::from_utf8(&key).unwrap_or_default().to_string(),
            }))
        })
        .collect::<serde_json::Value>();

    //titl
    Ok(response.to_string())
}
fn refresh_index_db() -> anyhow::Result<()> {
    //save db from all conversations
    let tree = sled::open(PATH_DIR_CHATGPT_CONFIG.get().unwrap().join("storage")).unwrap();
    let title_tree = tree.open_tree("title")?;
    tree.clear()?;
    tree.flush()?;
    title_tree.clear()?;
    title_tree.flush()?;

    //key: body, value: id
    let dir = unsafe { SAVING_DIRECTORY.clone() };
    let conversation_dir_path = std::path::Path::new(dir.as_str()).join(DIR_CONVERSATION);
    if let Ok(read_dir) = conversation_dir_path.read_dir() {
        for entry in read_dir.filter_map(|x| x.ok()) {
            let file_path_buff = entry.path().clone();
            let file_name = file_path_buff.file_name().unwrap().to_string_lossy();

            let file_data_str = std::fs::read_to_string(file_path_buff.as_path())?;
            // println!("file_data_str: {:#?}", file_data_str);
            let ai_conversation_obj: AIClientConversation =
                serde_json::from_str(file_data_str.as_str())?;
            // println!("file_name: {:#?}", file_name);
            tree.insert(
                std::fs::read_to_string(&file_path_buff)?.as_bytes(),
                file_name.as_bytes(),
            )?;
            title_tree.insert(file_name.as_bytes(), ai_conversation_obj.title.as_bytes())?;
        }
    }

    //key: id, value: title
    // let title_dir_path = std::path::Path::new(dir.as_str()).join(DIR_TITLE);
    // if let Ok(read_dir) = title_dir_path.read_dir() {
    //     for entry in read_dir.flatten() {
    //         let file_path = entry.path();
    //         let file_name = file_path.file_name().unwrap().to_string_lossy();
    //         let title = std::fs::read_to_string(&file_path)?;
    //         // println!("title {:#?}", title );
    //         title_tree.insert(file_name.as_bytes(), title.as_bytes())?;
    //     }
    // }
    title_tree.flush()?;
    tree.flush()?;
    Ok(())
}
//set title by chatGPT
async fn get_title(sentense: String) -> anyhow::Result<String> {
    let data: ChatApiSendMessage = ChatApiSendMessage {
        model: "gpt-4o-mini".into(),
        max_tokens: 1024,
        temperature: 0.5f32,
        messages: vec![ChatApiMessage {
            role: "user".into(),
            content: "Give it a simple title bellow text by own language. *Conditions=[Length<=20]. text>".to_string() + sentense.as_str(),
        }],
        stream: false,
    };

    let response = create_reqwest_client()
        .post(format!("{}/completions", "https://api.openai.com/v1/chat",).to_string())
        .json(&data)
        .timeout(Duration::from_secs(45))
        .send()
        .await
        .unwrap();

    if response.status() == 200 {
        let resData = response.json::<serde_json::Value>().await;
        let title = match resData {
            Ok(data) => {
                println!("data: {:#?}", data);
                let chatGptChunkData: ChatGptStopResponseData =
                    serde_json::from_value(data).unwrap();

                // data.get("choices").unwrap().[0].get("text").unwrap().as_str().unwrap().to_string()
                let choices = chatGptChunkData.choices.unwrap();
                Ok(choices[0].message.content.clone())
            }
            Err(err) => Err(anyhow::Error::msg("server error")),
        };
        title.map(|x| {
            x.replace("Title:", "")
                .replace("「", "")
                .replace("」", "")
                .replace("。", "")
                .replace("\"", "")
                .trim()
                .to_string()
        })
    } else {
        // println!(
        //     "response: {:#?}",
        //     response.json::<serde_json::Value>().await.ok()
        // );
        Err(anyhow::Error::msg("server error"))
    }
}

#[tauri::command]
async fn load_messages(app_handle: tauri::AppHandle, id: String) -> Result<String, String> {
    let dir = unsafe { SAVING_DIRECTORY.clone() };
    let file_path = std::path::Path::new(dir.as_str())
        .join(DIR_CONVERSATION)
        .join(id.clone());
    if file_path.exists() {
        let mut conversation = serde_json::from_str::<AIClientConversation>(
            std::fs::read_to_string(file_path)
                .unwrap_or_default()
                .as_str(),
        )
        .map_err(|x| x.to_string())?;

        for message in conversation.messages.iter_mut() {
            message.content_html = Some(markdown::to_html(message.content.as_str()));
        }

        Ok(serde_json::to_string(&conversation).unwrap())
    } else {
        Ok("".to_string())
    }
}

#[tauri::command]
async fn reflesh_threads(app_handle: tauri::AppHandle) -> Result<String, String> {
    let dir = unsafe { SAVING_DIRECTORY.clone() };
    let thread_path = std::path::Path::new(dir.as_str()).join(DIR_THREADS);
    if thread_path.exists() {
        if let Ok(read_dir) = thread_path.read_dir() {
            let data_vec = read_dir
                .filter_map(|entry| {
                    if let Ok(entry) = entry {
                        let datetime = Utc.timestamp_nanos(
                            entry
                                .metadata()
                                .unwrap()
                                .modified()
                                .unwrap()
                                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                                .unwrap()
                                .as_nanos() as i64,
                        );

                        let thread_file_string = std::fs::read_to_string(entry.path())
                            .map_err(|x| x.to_string())
                            .unwrap();
                        let mut threadData: ThreadData = if thread_file_string.is_empty() {
                            ThreadData::default()
                        } else {
                            serde_json::from_str(thread_file_string.as_str()).unwrap()
                        };
                        if threadData.created_at.is_none() {
                            threadData.time =
                                Some(datetime.format("%Y-%m-%d %H:%M:%S").to_string());
                        } else {
                            chrono::Utc
                                .timestamp_millis_opt(threadData.created_at.unwrap() * 1000)
                                .map(|x| {
                                    threadData.time =
                                        Some(x.format("%Y-%m-%d %H:%M:%S").to_string());
                                });
                        }
                        if threadData.id.is_none() {
                            threadData.id = Some(entry.file_name().to_string_lossy().to_string());
                        }
                        Some(threadData)
                    } else {
                        None
                    }
                })
                .collect::<Vec<ThreadData>>();
            return Ok(serde_json::to_string(&data_vec).unwrap());
        }
    }
    Err("".to_string())
}

#[tauri::command]
async fn reflesh_titles(app_handle: tauri::AppHandle) -> Result<String, String> {
    let dir = unsafe { SAVING_DIRECTORY.clone() };
    let conversation_path = std::path::Path::new(dir.as_str()).join(DIR_CONVERSATION);
    if conversation_path.exists() {
        if let Ok(read_dir) = conversation_path.read_dir() {
            let data_vec = read_dir
                .filter_map(|entry| {
                    if let Ok(entry) = entry {
                        let datetime = Utc.timestamp_nanos(
                            entry
                                .metadata()
                                .unwrap()
                                .modified()
                                .unwrap()
                                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                                .unwrap()
                                .as_nanos() as i64,
                        );
                        let mut ai_client_conversation_cata: AIClientConversation =
                            serde_json::from_str(
                                std::fs::read_to_string(entry.path()).unwrap().as_str(),
                            )
                            .unwrap();
                        ai_client_conversation_cata.time =
                            Some(datetime.format("%Y-%m-%d %H:%M:%S").to_string());
                        // let data = AIClientConversation {
                        //     name: std::fs::read_to_string(entry.path()).unwrap(),
                        //     id: entry.file_name().to_string_lossy().to_string(),
                        //     time: datetime.format("%Y-%m-%d %H:%M:%S").to_string(),
                        // };
                        Some(ai_client_conversation_cata)
                    } else {
                        None
                    }
                })
                .collect::<Vec<AIClientConversation>>();
            return Ok(serde_json::to_string(&data_vec).unwrap());
        }
    }
    Err("".to_string())
}

#[tauri::command]
async fn send_message_and_callback_stream(
    window: Window,
    app_handle: tauri::AppHandle,
    params: String,
    timeout_sec: Option<u64>,
) -> Result<String, String> {
    println!("timeout {:?}", timeout_sec);
    #[derive(Deserialize)]
    struct PostData {
        model: Option<String>,
        messages: Vec<ChatApiMessage>,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
        messageId: String,
    }
    let postData = serde_json::from_str::<PostData>(params.as_str()).unwrap();
    // let messages: Vec<ChatApiMessage> = serde_json::from_str(message.as_str()).unwrap();

    let data = ChatApiSendMessage {
        model: postData.model.unwrap_or("gpt-4o-mini".into()),
        max_tokens: postData.max_tokens.unwrap_or(2048),
        temperature: postData.temperature.unwrap_or(0.9f32),
        messages: postData.messages,
        stream: true,
    };

    println!("data: {:?}", data);

    // let rt = Runtime::new().unwrap();
    //     // Spawn a blocking function onto the runtime
    //     rt.block_on(async {

    let mut response_string = String::new();
    // let start_time = chrono::Utc::now();
    let mut prev_time = chrono::Utc::now();
    let messageId = postData.messageId.clone();

    // 非同期タスクとしてレスポンスの受信・処理を実行

    let response = create_reqwest_client()
        .post(format!("{}/completions", "https://api.openai.com/v1/chat",).to_string())
        .json(&data)
        .timeout(Duration::from_secs(timeout_sec.unwrap_or(45)))
        .send()
        .await
        .unwrap();

    let mut count = 0;
    let mut bytes_stream = response.bytes_stream();

    let task =
        tokio::spawn(async move {
            while let Some(chunk) = bytes_stream.next().await {
                match chunk {
                    Ok(chunk) => {
                        count += 1;
                        println!("count: {}", count);
                        for tmp_str in String::from_utf8(chunk.to_vec())
                            .unwrap()
                            .split("data:")
                            .filter(|&x| !x.replace("[DONE]", "").trim().is_empty())
                        {
                            // println!("trimed: {:?}", tmp_str.replace("[DONE]", "").trim());

                            let chatGptChunkData: ChatGptResponseData =
                                serde_json::from_str(tmp_str.replace("[DONE]", "").trim()).unwrap();

                            if let Some(error) = chatGptChunkData.error {
                                window
                                    .emit("stream_error", serde_json::to_string(&error).unwrap())
                                    .unwrap();
                            } else if let Some(choices) = chatGptChunkData.choices {
                                for choice in choices {
                                    if let Some(content) = choice.delta.content {
                                        //emit every more 3 seconds.
                                        let now = chrono::Utc::now();
                                        let duration = now - prev_time;
                                        response_string.push_str(&content);

                                        println!(
                                            "markdown::to_html(&response_string):{:?}",
                                            markdown::to_html(&response_string)
                                        );
                                        if duration.gt(&chrono::Duration::milliseconds(200)) {
                                            prev_time = now;
                                            window
                                        .emit("stream_chunk", serde_json::json!({
                                            "messageId": messageId.clone(), 
                                            "response": response_string.clone(), 
                                            // "responseHtml": markdown::to_html(&response_string)
                                            "responseHtml": markdown::to_html(&response_string)
                                        }))
                                        .unwrap();
                                        }
                                    }
                                    if let Some(finish_reason) = choice.finish_reason {
                                        println!("finish_reason: {:?}", finish_reason);
                                        // println!(
                                        //     "finish... markdown::to_html(&response_string):{:?}",
                                        //     markdown::to_html(&response_string)
                                        // );
                                        window
                                        .emit("finish_chunks", serde_json::json!({
                                            "messageId": messageId.clone(), 
                                            "response": response_string.clone(), 
                                            // "responseHtml":  markdown::to_html(&response_string)
                                            "responseHtml":  markdown::to_html(&response_string)
                                        }))
                                        .unwrap();
                                    }
                                }
                            }
                        }
                    }
                    Err(err) => {
                        if err.is_timeout() {
                            //timeout
                            println!("timeout!");
                            window.emit("timeout_stream", messageId.clone()).unwrap();
                        } else {
                            //TODO each error.
                            println!("other err! {:?}", err);
                            window.emit("timeout_stream", messageId.clone()).unwrap();
                        }
                        break;
                    }
                }
            }
            future::ready(())
        });
    // task::spawn(async move {
    //     let mut chars = String::new();
    //     while true  {
    //         //generate random chars
    //         let mut rng = rand::thread_rng();
    //         let rng: String = (0..20).into_iter()
    //             .map(|_| rng.sample(Alphanumeric))
    //             .map(char::from)
    //             .take(10)
    //             .collect();
    //         chars.push_str((rng + "\\n<br>\\r\\n").as_str());
    //         window
    //         .emit("stream_chunk", serde_json::json!({"messageId":postData.messageId, "response": chars.clone(), "responseHtml": markdown::to_html(&chars)}))
    //         .unwrap();
    //         // std::time::sleep(Duration::from_secs(1));
    //         //sleep 10 seconds
    //         std::thread::sleep(std::time::Duration::from_secs(1));
    //     }
    // });
    Ok("stream go on".into())
}

fn init_config(app: &tauri::App) -> anyhow::Result<()> {
    let chat_gpt_config_dir = PATH_DIR_CHATGPT_CONFIG.get_or_init(|| {
        app.path().app_config_dir()
            .unwrap()
            .join("chatGPT")
    });

    let config_toml_file_path = chat_gpt_config_dir.join("config.toml");

    println!("config_toml_file_path: {:#?}", config_toml_file_path);

    if !config_toml_file_path.exists() {
        std::fs::create_dir_all(chat_gpt_config_dir.clone()).unwrap();
        let mut f = File::create(config_toml_file_path.clone()).unwrap();
        let config_file = toml::to_string(&Config::default()).unwrap();
        f.write_all(config_file.as_bytes())
            .expect("Unable to write config data");
    } else {
        let f = File::open(config_toml_file_path.clone()).expect("can't open config file.");
        let mut buf_reader = BufReader::new(f);
        let mut config_data: Vec<u8> = vec![];
        buf_reader.read_to_end(&mut config_data).unwrap();
        let config: Config =
            toml::from_str(String::from_utf8(config_data).unwrap().as_str()).unwrap();
        unsafe {
            // API_KEY = config.keys.chatgpt.unwrap_or_default();
            SAVING_DIRECTORY = config.saving_directory.unwrap_or_default();
        }
        {
            let mut key = API_KEY.write().unwrap();
            *key = config.keys.chatgpt.unwrap_or_default();
        }
    }

    refresh_index_db().unwrap();

    Ok(())
}
fn main() {
    infrastracture::logging::init_logging();
    // let main_page = CustomMenuItem::new("main".to_string(), "Main");
    // let settings = CustomMenuItem::new("settings".to_string(), "Settings");
    // let assistants = CustomMenuItem::new("assistants".to_string(), "Assistants");
    // let samples = CustomMenuItem::new("samples".to_string(), "Samples");
    // let open_ai_files = CustomMenuItem::new("open_ai_files".to_string(), "OpenAIFiles");

    // let submenu = Submenu::new(
    //     "Menu",
    //     Menu::new()
    //         .add_item(main_page)
    //         .add_item(assistants)
    //         .add_item(open_ai_files)
    //         .add_item(samples)
    //         .add_item(settings),
    // );
    // let menu = Menu::new().add_submenu(submenu);
    let context = tauri::generate_context!();
    // let menu = tauri::Menu::os_default(&context.package_info().name).add_submenu(submenu);
    
    // let dir = "/home/masatoyuna/nextcloud/chatgpt".to_string();
    // let conversation_dir_path = std::path::Path::new(dir.as_str()).join(DIR_CONVERSATION);
    // let title_dir_path = std::path::Path::new(dir.as_str()).join("titles");
    // if let Ok(read_dir) = conversation_dir_path.read_dir() {
    //     for entry in read_dir.filter_map(|x| x.ok()) {

    //         let file_path_buff = entry.path().clone();
    //         let file_name = file_path_buff.file_name().unwrap().to_string_lossy();

    //         let conversation_list_str = std::fs::read_to_string(file_path_buff.as_path()).unwrap();

    //         let title_path = title_dir_path.join(file_name.as_ref());
    //         // println!("title_path: {:#?}", title_path);
    //         let title = std::fs::read_to_string(title_path.as_path()).unwrap();

    //         // println!("title: {:#?}", title);
    //         // println!("file_data_str: {:#?}", conversation_list_str);
    //         let new_obj = AIClientConversation{
    //             id: file_name.to_string(),
    //             title: title,
    //             messages: serde_json::from_str::<Vec<ChatApiMessageWithHtml>>(conversation_list_str.as_str()).unwrap(),
    //             ..Default::default()
    //         };
    //         println!("ob: {:#?}", new_obj);
    //         std::fs::write(file_path_buff.as_path(), serde_json::to_string(&new_obj).unwrap()).unwrap();
    //     }
    // }

    // return;
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        // .menu(menu)
        // .on_menu_event(|event| match event.menu_item_id() {
        //     "settings" => {
        //         event.window().emit("open_settings", "").unwrap();
        //     }
        //     "assistants" => {
        //         event.window().emit("open_assistants", "").unwrap();
        //     }
        //     "open_ai_files" => {
        //         event.window().emit("open_open_ai_files", "").unwrap();
        //     }
        //     "samples" => {
        //         event.window().emit("open_samples", "").unwrap();
        //     }
        //     "main" => {
        //         event.window().emit("open_main", "").unwrap();
        //     }
        //     _ => {
        //         println!("menu event: {:?}", event.menu_item_id());
        //     }
        // })
        .invoke_handler(tauri::generate_handler![
            save_chat,
            send_message_and_callback_stream,
            set_api_key,
            get_api_key,
            reflesh_titles,
            reflesh_threads,
            open_ai_files::reflesh_openai_files,
            open_ai_files::reflesh_vectors,
            open_ai_files::upload_files,
            load_messages,
            delete_message,
            delete_thread,
            open_ai_files::delete_openai_file,
            open_ai_files::delete_vector,
            open_ai_files::make_vector,
            change_message,
            search_conversations,
            reflesh_index,
            assistants::make_assistant,
            assistants::delete_assistant,
            assistants::reflesh_assistants,
            assistants::make_new_thread,
            assistants_example::assistents_test,
            assistants_stream::assistents_stream_test,
            assistants_file_search::assistents_file_search_test,
            assistants_code_interpreter::assistents_code_interpreter_test,
            assistants_audio_transcribe::assistants_audio_transcribe_test,
            assistants_audio_translate::assistants_audio_translate_test,
            assistants_audio_speech::assistants_audio_speech_test,
            assistants_vision_chat::assistants_vision_chat_test,
            assistants_tool_calls::assistants_tool_calls_test,
            audio::audio_transcribe,
            embedding::embedding_test,
            chat_completion::start_chat,
        ])
        .setup(|app| {
            init_config(app).expect("config init error");

            //setup menu
            // let main_page = CustomMenuItem::new("main".to_string(), "Main");
            // let settings = CustomMenuItem::new("settings".to_string(), "Settings");
            // let assistants = CustomMenuItem::new("assistants".to_string(), "Assistants");
            // let samples = CustomMenuItem::new("samples".to_string(), "Samples");
            // let open_ai_files = CustomMenuItem::new("open_ai_files".to_string(), "OpenAIFiles");
            let main_page = MenuItemBuilder::with_id("main", "Main").build(app)?;
            let settings = MenuItemBuilder::with_id("settings", "Settings").build(app)?;
            let assistants = MenuItemBuilder::with_id("assistants", "Assistants").build(app)?;
            let samples = MenuItemBuilder::with_id("samples", "Samples").build(app)?;
            let open_ai_files = MenuItemBuilder::with_id("open_ai_files", "OpenAIFiles").build(app)?;

            // let submenu = Submenu::new(
            //     "Menu",
            //     Menu::new()
            //         .add_item(main_page)
            //         .add_item(assistants)
            //         .add_item(open_ai_files)
            //         .add_item(samples)
            //         .add_item(settings),
            // );
            // let menu = Menu::new().add_submenu(submenu);
            // let context = tauri::generate_context!();
            // let menu = tauri::Menu::os_default(&context.package_info().name).add_submenu(submenu);
            let submenu = SubmenuBuilder::new(app, "Menu").items(&[&main_page, &settings, &assistants, &samples, &open_ai_files]).build()?;
            
            let toggle = MenuItemBuilder::with_id("toggle", "Toggle").build(app)?;
                let check = CheckMenuItemBuilder::new("Mark").build(app)?;
                // let copy=PredefinedMenuItem::copy(app, None)?;
                let quit=PredefinedMenuItem::quit(app, Some("Quit"))?;
                let close =PredefinedMenuItem::close_window(app, Some("Close"))?;
                let menu = MenuBuilder::new(app).items(&[&quit, &close, &submenu]).build()?;

                app.set_menu(menu)?;
                app.on_menu_event(move |app, event| {
                    if event.id() == check.id() {
                        println!("`check` triggered, do something! is checked? {}", check.is_checked().unwrap());
                    } else if event.id() == "toggle" {
                        println!("toggle triggered!");
                    }
                    if let MenuId(id) = event.id() {
                        match id.as_str() {
                            "settings" => {
                                app.emit("open_settings", "").unwrap();
                            }
                            "assistants" => {
                                app.emit("open_assistants", "").unwrap();
                            }
                            "open_ai_files" => {
                                app.emit("open_open_ai_files", "").unwrap();
                            }
                            "samples" => {
                                app.emit("open_samples", "").unwrap();
                            }
                            "main" => {
                                app.emit("open_main", "").unwrap();
                            }
                            _ => {
                                println!("menu event: {:?}", id);
                            }
                        }
                    }
                });

                
            Ok(())
        })
        .run(context)
        .expect("error while running tauri application");
}
