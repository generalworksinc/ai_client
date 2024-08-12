use std::{
    fs::File,
    path::{Path},
};

use crate::API_KEY;
use anyhow::Result;
use async_openai::{config::OpenAIConfig, Client};
use base64::prelude::*;
use std::io::prelude::*;

const OPENAI_THREAD_HEAD_WORD: &str = "thread_";
const OPENAI_ASSISTANT_HEAD_WORD: &str = "asst_";

pub fn create_client() -> Result<Client<OpenAIConfig>> {
    let api_key = API_KEY
        .read()
        .map_err(|e| anyhow::anyhow!("failed to read API_KEY: {:?}", e))?;
    Ok(Client::with_config(
        OpenAIConfig::new().with_api_key(api_key.clone()),
    ))
}

// pub fn get_thread_and_assistant_id(id: &str) -> (String, String) {
//     //thread_xxxxxxassistant_yyyyyy -> (xxxxxx, yyyyyy)
//     if id.starts_with(OPENAI_THREAD_HEAD_WORD) {
//         let parts: Vec<&str> = id.split(OPENAI_THREAD_HEAD_WORD).collect();
//         let tmp_list = parts[1]
//             .split(OPENAI_ASSISTANT_HEAD_WORD)
//             .collect::<Vec<&str>>();
//         let thread_id = tmp_list[0];
//         let assistant_id = if tmp_list.len() >= 2 { tmp_list[1] } else { "" };

//         (
//             OPENAI_THREAD_HEAD_WORD.to_string() + thread_id,
//             OPENAI_ASSISTANT_HEAD_WORD.to_string() + assistant_id,
//         )
//     } else {
//         ("".to_string(), "".to_string())
//     }
// }

pub fn is_thread(id: &str) -> bool {
    id.starts_with(OPENAI_THREAD_HEAD_WORD)
}

pub fn get_file_base64(file_path: &Path) -> Result<String> {
    let mut file = File::open(file_path)?;
    let mut buffer = Vec::new();

    // ファイルの内容を読み込む
    file.read_to_end(&mut buffer)?;

    // MIMEタイプを取得
    let mime_type = mime_guess::from_path(file_path).first_or_octet_stream();

    // Base64エンコード
    let base64_data = BASE64_STANDARD.encode(&buffer);
    // データURL形式に整形
    Ok(format!("data:{};base64,{}", mime_type, base64_data))
}

pub fn get_file_binary(file_path: &Path) -> Result<Vec<u8>> {
    let mut file = File::open(file_path)?;
    let mut buffer = Vec::new();

    // ファイルの内容を読み込む
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}
