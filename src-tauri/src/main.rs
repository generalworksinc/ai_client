// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use futures::future;
use futures::stream::{self, StreamExt};
use rand::distributions::Alphanumeric;
use serde_json::Value;
use std::f32::consts::E;
use std::path::{PathBuf,Path};
use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::io::prelude::*;
use std::time::Duration;
use tauri::{CustomMenuItem, Menu, MenuItem, Submenu};
use tauri::{Manager, Window, WindowUrl};
// use futures_util::stream::StreamExt;
use tokio::runtime::Runtime;
use tokio::task;
use rand::prelude::*;


use reqwest::{header, multipart, Body, Client};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, TimeZone};
use once_cell::sync::{Lazy, OnceCell};
static mut API_KEY: String = String::new();
static mut SAVING_DIRECTORY: String = String::new();
const DIR_TITLE: &str = "titles";
const DIR_CONVERSATION: &str = "conversations";

pub static PATH_DIR_CHATGPT_CONFIG:  OnceCell<PathBuf> = OnceCell::new();

pub fn create_client() -> reqwest::Client {
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
            header::HeaderValue::from_str(format!("Bearer {}", API_KEY).as_str()).unwrap(),
        );
    }

    let client = reqwest::Client::builder()
        .use_rustls_tls()
        .default_headers(headers)
        .build()
        .unwrap();
    return client;
}

////////ChatGPT API Response /////////////////////////////////////////////////////
#[derive(Serialize, Deserialize, Debug)]
struct ChatApiMessage {
    role: String,
    content: String,
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

#[derive(Deserialize, Serialize)]
struct TitleData {
    id: String,
    name: String,
    time: String,
}

#[tauri::command]
async fn set_api_key(app_handle: tauri::AppHandle, api_key: String, saving_directory: String) -> Result<String, String> {
    let chat_gpt_config_dir = PATH_DIR_CHATGPT_CONFIG.get().unwrap();

    let config_toml_file_path = chat_gpt_config_dir.join("config.toml");

    if (!config_toml_file_path.exists()) {
        std::fs::create_dir_all(chat_gpt_config_dir.clone()).unwrap();
        let mut f = File::create(config_toml_file_path.clone()).unwrap();
        let config_file = toml::to_string(&Config::default()).unwrap();
        f.write_all(config_file.as_bytes())
            .expect("Unable to write config data");
    }

    if let Ok(mut f) = File::open(config_toml_file_path.clone()) {
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
        unsafe {
            API_KEY = api_key;
        }
        return Ok("save config data.".into());
    }

    return Ok("can't save config data.".into());
}
#[tauri::command]
async fn get_api_key(app_handle: tauri::AppHandle) -> Result<String, String> {
    let res = unsafe { 
        serde_json::json!({
            "apiKey": API_KEY.clone(),
            "savingDirectory": SAVING_DIRECTORY.clone(),
        })
    };
    Ok(res.to_string())
}

#[tauri::command]
async fn change_message(app_handle: tauri::AppHandle, id: String, name: String) -> Result<String, String> {
    let dir = unsafe{SAVING_DIRECTORY.clone()};
    let file_path_title = std::path::Path::new(dir.as_str()).join(DIR_TITLE).join(id.clone());
    
    if file_path_title.exists() {
        std::fs::write(file_path_title, name).map_err(|err| err.to_string())?;
        //更新
        // title_f.write_all(title.as_bytes()).unwrap();
    }
    Ok("変更しました".to_string())
}
#[tauri::command]
async fn delete_message(app_handle: tauri::AppHandle, id: String) -> Result<String, String> {
    let dir = unsafe{SAVING_DIRECTORY.clone()};
    let file_path_conversation = std::path::Path::new(dir.as_str()).join(DIR_CONVERSATION).join(id.clone());
    let file_path_title = std::path::Path::new(dir.as_str()).join(DIR_TITLE).join(id.clone());
    if file_path_conversation.exists() {
        //削除
        std::fs::remove_file(file_path_conversation).map_err(|x| x.to_string())?;
    }
    if file_path_title.exists() {
        //削除
        std::fs::remove_file(file_path_title).map_err(|x| x.to_string())?;
    }
    Ok("削除しました".to_string())
}

#[tauri::command]
async fn send_message(app_handle: tauri::AppHandle, message: String) -> Result<String, String> {
    println!("call send_message! message:{message}");
    let messages: Vec<ChatApiMessage> = serde_json::from_str(message.as_str()).unwrap();

    let data = ChatApiSendMessage {
        model: "gpt-3.5-turbo".into(),
        max_tokens: 1024,
        temperature: 0.9f32,
        messages: messages,
        stream: false,
    };

    let response = create_client()
        .post(format!("{}/completions", "https://api.openai.com/v1/chat",).to_string())
        .json(&data)
        .timeout(Duration::from_secs(45))
        .send()
        .await
        .unwrap();

    if response.status() == 200 {
        let resData = response.json::<serde_json::Value>().await;
        match resData {
            Ok(data) => return Ok(data.to_string()),
            Err(err) => {
                return Err("server error".into());
            }
        }
    } else {
        println!(
            "response: {:#?}",
            response.json::<serde_json::Value>().await.ok()
        );
        return Err("server error".into());
    }
}

#[tauri::command]
async fn save_chat(
    app_handle: tauri::AppHandle,
    params: String,
) -> Result<String, String> {
    #[derive(Deserialize, Serialize)]
    struct PostData {
        data: Vec<ChatApiMessage>,
        // data: String,
        id: Option<String>,
    }
    println!("params: {:#?}", params);
    let postData = serde_json::from_str::<PostData>(params.as_str()).unwrap();
    // let messages = serde_json::from_str::<Vec<ChatApiMessage>>(postData.data.as_str()).unwrap();

    // write_message into file.
    let id = if let Some(id) = postData.id {
        id
    } else {
        uuid::Uuid::new_v4().to_string()
    };
    // write_id and conversasion.
    let dir = unsafe{SAVING_DIRECTORY.clone()};
    let content_dir_path = std::path::Path::new(dir.as_str()).join(DIR_CONVERSATION);

    if !content_dir_path.exists() {
        if let Err(_) = std::fs::create_dir_all(content_dir_path.as_path()) {
            return Err("can't create title directory.".into());
        }
    }
    let file_path = content_dir_path.join(id.clone());
    let mut f = File::create(file_path).unwrap();
    f.write_all(serde_json::to_string(&postData.data).unwrap().as_bytes()).unwrap();

    // write_id and title.
    let title_dir_path = std::path::Path::new(dir.as_str()).join(DIR_TITLE);
    if !title_dir_path.exists() {
        if let Err(_) = std::fs::create_dir_all(title_dir_path.as_path()) {
            return Err("can't create title directory.".into());
        }
    }
    let title_file = title_dir_path.join(id.clone());
    
    //make file title
    let mut title_content = "".to_string();
    for message in postData.data {
        println!("message: {:#?}", message);
        if message.role == "user" {
            title_content += message.content.as_str();
            break;
        } else if message.role == "assistant" {
            title_content += message.content.as_str();
            break;
        }
    }
    let mut title_f = File::create(title_file).unwrap();
    println!("title_content: {:#?}", title_content.clone());
    if title_content.len() > 30 {
        match get_title(title_content.clone()).await {
            Ok(title) => {
                println!("title: {:#?}", title);
                title_f.write_all(title.as_bytes()).unwrap();
            }
            Err(err) => {
                println!("err: {:#?}", err);
                // title_f.write_all(title_content.as_bytes()).unwrap();
                return Err("".to_string())
            }
        }
    } else {
        title_f.write_all(title_content.as_bytes()).unwrap();
    }
    
    refresh_index_db().unwrap();
    Ok("".to_string())
}

#[tauri::command]
async fn reflesh_index(app_handle: tauri::AppHandle) -> Result<String, String> { 
    refresh_index_db().unwrap();
    Ok("".to_string())
}
fn from_u8_to_str(buf: &[u8]) -> &str{
    let s = match std::str::from_utf8(buf) {
      Ok(v) => v,
      Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };
    s
  }
#[tauri::command]
async fn search_conversations(app_handle: tauri::AppHandle, word: String) -> Result<String, String> {
    
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
    let response = tree.iter().flatten().filter_map(|(key, value)| {
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
    }).collect::<serde_json::Value>();

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
    let dir = unsafe{SAVING_DIRECTORY.clone()};
    let conversation_dir_path = std::path::Path::new(dir.as_str()).join(DIR_CONVERSATION);
    if let Ok(read_dir) = conversation_dir_path.read_dir() {
        for entry in read_dir.flatten() {
            let file_path = entry.path();
            let file_name = file_path.file_name().unwrap().to_string_lossy();
            println!("file_name: {:#?}", file_name);
            tree.insert( std::fs::read_to_string(&file_path)?.as_bytes(), file_name.as_bytes())?;
        }
    }

    //key: id, value: title
    let title_dir_path = std::path::Path::new(dir.as_str()).join(DIR_TITLE);
    if let Ok(read_dir) = title_dir_path.read_dir() {
        for entry in read_dir.flatten() {
            let file_path = entry.path();
            let file_name = file_path.file_name().unwrap().to_string_lossy();
            let title = std::fs::read_to_string(&file_path)?;
            println!("title {:#?}", title );
            title_tree.insert(  file_name.as_bytes(), title.as_bytes())?;
        }
    }
    title_tree.flush()?;
    tree.flush()?;
    Ok(())
}
//set title by chatGPT
async fn get_title(sentense: String) -> anyhow::Result<String> {
    let data = ChatApiSendMessage {
        model: "gpt-3.5-turbo".into(),
        max_tokens: 1024,
        temperature: 0.5f32,
        messages: vec![ChatApiMessage {
            role: "user".into(),
            content: "Give it a simple title bellow text by own language. *Conditions=[Length<=20]. text>".to_string() + sentense.as_str(),
        }],
        stream: false,
    };

    let response = create_client()
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
            },
            Err(err) => {
                Err(anyhow::Error::msg("server error"))
            }
        };
        title.map(|x| x.replace("Title:","").replace("「","").replace("」","").replace("。","").replace("\"","").trim().to_string())
    } else {
        // println!(
        //     "response: {:#?}",
        //     response.json::<serde_json::Value>().await.ok()
        // );
        return Err(anyhow::Error::msg("server error"));
    }
}

#[tauri::command]
async fn load_messages(
    app_handle: tauri::AppHandle,
    id: String,
) -> Result<String, String> {
    let dir = unsafe{SAVING_DIRECTORY.clone()};
    let file_path = std::path::Path::new(dir.as_str()).join(DIR_CONVERSATION).join(id.clone());
    if file_path.exists() {
        let mut messages = serde_json::from_str::<Vec<ChatApiMessageWithHtml>>(std::fs::read_to_string(file_path).unwrap_or_default().as_str()).unwrap();
        for message in messages.iter_mut() {
            message.content_html = Some(markdown::to_html(message.content.as_str()).into());
        }
        
        Ok(serde_json::to_string(&messages).unwrap())
    } else {
        Ok("".to_string())
    }
}

#[tauri::command]
async fn reflesh_titles(
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    let dir = unsafe{SAVING_DIRECTORY.clone()};
    let title_path = std::path::Path::new(dir.as_str()).join(DIR_TITLE);
    if title_path.exists() {
        if let Ok(read_dir) = title_path.read_dir() {
            let data_vec = read_dir.filter_map(|entry| {
                if let Ok(entry) = entry {
                    let datetime = Utc.timestamp_nanos(
                            entry.metadata().unwrap().modified().unwrap().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap().as_nanos() as i64
                        );
                    
                    let data = TitleData{
                        name: std::fs::read_to_string(entry.path()).unwrap(),
                        id: entry.file_name().to_string_lossy().to_string(),
                        time: datetime.format("%Y-%m-%d %H:%M:%S").to_string(),
                    };
                    Some(data)
                } else {
                    None
                }
            }).collect::<Vec<TitleData>>();
            return Ok(serde_json::to_string(&data_vec).unwrap());
        }
    }
    return Err("".to_string());
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
        model: postData.model.unwrap_or("gpt-3.5-turbo".into()),
        max_tokens: postData.max_tokens.unwrap_or(1024),
        temperature: postData.temperature.unwrap_or(0.9f32),
        messages: postData.messages,
        stream: true,
    };

    println!("data: {:?}", data);
    
    // let rt = Runtime::new().unwrap();
    //     // Spawn a blocking function onto the runtime
    //     rt.block_on(async {
            

    let mut response_string = String::new();
    let start_time = chrono::Utc::now();
    let mut prev_time = chrono::Utc::now();
    let messageId = postData.messageId.clone();
    
    // 非同期タスクとしてレスポンスの受信・処理を実行
    
    let response = create_client()
    .post(format!("{}/completions", "https://api.openai.com/v1/chat",).to_string())
    .json(&data)
    .timeout(Duration::from_secs(timeout_sec.unwrap_or(45)))
    .send()
    .await
    .unwrap();

    let mut count = 0;
    let mut bytes_stream = response.bytes_stream();

    let task = tokio::spawn(async move {
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


                                    //generate random chars
                                    // let mut rng = rand::thread_rng();
                                    // let rng: String = (0..20).into_iter()
                                    // .map(|_| rng.sample(Alphanumeric))
                                    // .map(char::from)
                                    // .take(10)
                                    // .collect();
                                    // response_string.push_str((rng + "\\n<br>\\r\\n").as_str());
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
                },
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
    let chat_gpt_config_dir = PATH_DIR_CHATGPT_CONFIG
        .get_or_init(|| app.path_resolver().app_config_dir().unwrap().join("chatGPT"));

    let config_toml_file_path = chat_gpt_config_dir.join("config.toml");

    println!("config_toml_file_path: {:#?}", config_toml_file_path);

    if (!config_toml_file_path.exists()) {
        std::fs::create_dir_all(chat_gpt_config_dir.clone()).unwrap();
        let mut f = File::create(config_toml_file_path.clone()).unwrap();
        let config_file = toml::to_string(&Config::default()).unwrap();
        f.write_all(config_file.as_bytes())
            .expect("Unable to write config data");
    } else {
        let mut f = File::open(config_toml_file_path.clone()).expect("can't open config file.");
        let mut buf_reader = BufReader::new(f);
        let mut config_data: Vec<u8> = vec![];
        buf_reader.read_to_end(&mut config_data).unwrap();
        let mut config: Config =
            toml::from_str(String::from_utf8(config_data).unwrap().as_str()).unwrap();
        unsafe {
            API_KEY = config.keys.chatgpt.unwrap_or_default();
            SAVING_DIRECTORY = config.saving_directory.unwrap_or_default();
        }
    }

    refresh_index_db().unwrap();
    
    Ok(())
}
fn main() {
    let main_page = CustomMenuItem::new("main".to_string(), "Main");
    let settings = CustomMenuItem::new("settings".to_string(), "Settings");
    let submenu = Submenu::new("Menu", Menu::new().add_item(main_page).add_item(settings));
    // let menu = Menu::new().add_submenu(submenu);
    let context = tauri::generate_context!();
    let menu = tauri::Menu::os_default(&context.package_info().name).add_submenu(submenu);

    tauri::Builder::default()
        .menu(menu)
        .on_menu_event(|event| match event.menu_item_id() {
            "settings" => {
                event.window().emit("open_settings", "").unwrap();
            }
            "main" => {
                event.window().emit("open_main", "").unwrap();
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            save_chat,
            send_message,
            send_message_and_callback_stream,
            set_api_key,
            get_api_key,
            reflesh_titles,
            load_messages,
            delete_message,
            change_message,
            search_conversations,
            reflesh_index,
        ])
        .setup(|app| {
            init_config(&app).expect("config init error");
            Ok(())
        })
        .run(context)
        .expect("error while running tauri application");
}
