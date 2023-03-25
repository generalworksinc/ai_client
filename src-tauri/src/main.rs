// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use futures::future;
use futures::stream::{self, StreamExt};
use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::time::Duration;
use tauri::{CustomMenuItem, Menu, MenuItem, Submenu};
use tauri::{Manager, Window, WindowUrl};
// use futures_util::stream::StreamExt;

use reqwest::{header, multipart, Body, Client};
use serde::{Deserialize, Serialize};

static mut API_KEY: String = String::new();

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
}

#[derive(Serialize, Deserialize, Default)]
struct Keys {
    pub chatgpt: Option<String>,
    //    travis: Option<String>,
}

#[tauri::command]
async fn set_api_key(app_handle: tauri::AppHandle, value: String) -> Result<String, String> {
    let chat_gpt_config_dir = app_handle
        .path_resolver()
        .app_config_dir()
        .unwrap()
        .join("chatGPT");

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

        config.keys.chatgpt = Some(value.clone());
        let config_file = toml::to_string(&config).unwrap();
        let mut f = File::create(config_toml_file_path.clone()).unwrap();
        f.write_all(config_file.as_bytes())
            .expect("Unable to write config data");
        unsafe {
            API_KEY = value;
        }
        return Ok("save config data.".into());
    }

    return Ok("can't save config data.".into());
}
#[tauri::command]
async fn get_api_key(app_handle: tauri::AppHandle) -> Result<String, String> {
    unsafe { Ok(API_KEY.clone()) }
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
                return Err("server エラー".into());
            }
        }
    } else {
        println!(
            "response: {:#?}",
            response.json::<serde_json::Value>().await.ok()
        );
        return Err("server エラー".into());
    }
}

#[tauri::command]
async fn send_message_and_callback_stream(
    window: Window,
    app_handle: tauri::AppHandle,
    params: String,
) -> Result<String, String> {
    #[derive(Deserialize)]
    struct PostData {
        model: Option<String>,
        messages: Vec<ChatApiMessage>,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
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
    let response = create_client()
        .post(format!("{}/completions", "https://api.openai.com/v1/chat",).to_string())
        .json(&data)
        .timeout(Duration::from_secs(45))
        .send()
        .await
        .unwrap();

    let mut response_string = String::new();
    let for_each_val = response
        .bytes_stream()
        // .filter_map(|x| x.ok())
        .for_each(|chunk| {
            let chunk = chunk.unwrap();
            for tmp_str in String::from_utf8(chunk.to_vec())
                .unwrap()
                .split("data:")
                .filter(|&x| !x.replace("[DONE]", "").trim().is_empty())
            {
                println!("trimed: {:?}", tmp_str.replace("[DONE]", "").trim());

                let chatGptChunkData: ChatGptResponseData =
                    serde_json::from_str(tmp_str.replace("[DONE]", "").trim()).unwrap();

                if let Some(error) = chatGptChunkData.error {
                    window
                        .emit("stream_error", serde_json::to_string(&error).unwrap())
                        .unwrap();
                } else if let Some(choices) = chatGptChunkData.choices {
                    for choice in choices {
                        if let Some(content) = choice.delta.content {
                            response_string.push_str(&content);
                            println!(
                                "markdown::to_html(&response_string):{:?}",
                                markdown::to_html(&response_string)
                            );
                            window
                                .emit("stream_chunk", markdown::to_html(&response_string))
                                .unwrap();
                        }
                        if let Some(finish_reason) = choice.finish_reason {
                            println!("finish_reason: {:?}", finish_reason);
                            println!(
                                "finish... markdown::to_html(&response_string):{:?}",
                                markdown::to_html(&response_string)
                            );
                            window
                                .emit("finish_chunks", response_string.clone())
                                .unwrap();
                        }
                    }
                }
            }
            future::ready(())
        })
        .await;
    Ok("".into())
}

fn init_config(app: &tauri::App) -> anyhow::Result<()> {
    let chat_gpt_config_dir = app
        .path_resolver()
        .app_config_dir()
        .unwrap()
        .join("chatGPT");

    let config_toml_file_path = chat_gpt_config_dir.join("config.toml");

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
        }
    }
    Ok(())
}
fn main() {
    let main_page = CustomMenuItem::new("main".to_string(), "Main");
    let settings = CustomMenuItem::new("settings".to_string(), "Settings");
    let submenu = Submenu::new("Menu", Menu::new().add_item(main_page).add_item(settings));
    let menu = Menu::new().add_submenu(submenu);

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
            send_message,
            send_message_and_callback_stream,
            set_api_key,
            get_api_key,
        ])
        .setup(|app| {
            init_config(&app).expect("config init error");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
