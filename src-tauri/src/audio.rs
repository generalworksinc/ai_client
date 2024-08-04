use crate::util;
use base64::prelude::*;
use base64::{
    engine::general_purpose::{STANDARD, URL_SAFE},
    Engine as _,
};
use serde::Deserialize;
use serde_json::json;
use tauri::Window;

use crate::API_KEY;
use async_openai::{
    config::OpenAIConfig,
    types::{
        AudioInput, AudioResponseFormat, CreateTranscriptionRequestArgs, TimestampGranularity,
    },
    Client,
};
use futures::StreamExt;

#[tauri::command]
pub async fn audio_transcribe(
    window: Window,
    app_handle: tauri::AppHandle,
    filebody: String,
    filename: String,
) -> Result<String, String> {
    println!("call audio_transcribe: {:#?}", filename);
    match audio_transcribe_exec(filename.as_str(), filebody.as_str())
        .await
        .map_err(|e| format!("{:?}", e))
    {
        Ok(value) => Ok(serde_json::to_string(&value).map_err(|x| x.to_string())?),
        Err(err) => {
            eprintln!("Error: {:?}", err);
            Err(err)
        }
    }
}

async fn audio_transcribe_exec(
    file_name: &str,
    file_body: &str,
) -> anyhow::Result<serde_json::Value> {
    // transcribe_json().await?;
    transcribe_verbose_json(file_name, file_body).await
}

async fn transcribe_verbose_json(
    file_name: &str,
    file_body: &str,
) -> Result<serde_json::Value, anyhow::Error> {
    let mut file_binary: Vec<u8>;
    // let stan = base64::engine::general_purpose::STANDARD;

    if let Some((file_type, file_body)) = file_body.clone().split_once("base64,") {
        file_binary = BASE64_STANDARD.decode(file_body)?;
    } else {
        return Err(anyhow::anyhow!("Invalid file format"));
    }
    // } else {
    //     return Err(anyhow::anyhow!("Invalid file format"));
    // }
    println!("file_binary len: {:?}", file_binary.len());

    let client = util::create_client()?;
    let bytes = bytes::Bytes::from(file_binary);
    let audio_input = AudioInput::from_bytes(file_name.to_string(), bytes);

    let request = CreateTranscriptionRequestArgs::default()
        .file(audio_input)
        .model("whisper-1")
        .response_format(AudioResponseFormat::VerboseJson)
        .timestamp_granularities(vec![
            TimestampGranularity::Word,
            TimestampGranularity::Segment,
        ])
        .build()?;

    let response = client.audio().transcribe_verbose_json(request).await?;

    println!("{}", response.text);
    if let Some(words) = &response.words {
        println!("- {} words", words.len());
    }
    if let Some(segments) = &response.segments {
        println!("- {} segments", segments.len());
    }

    Ok(json!({
        "text": response.text,
        "words": response.words,
        "segments": response.segments,
    }))
}

// async fn transcribe_json() -> Result<(), anyhow::Error> {
//     let client = util::create_client()?;
//     // Credits and Source for audio: https://www.youtube.com/watch?v=oQnDVqGIv4s
//     let request = CreateTranscriptionRequestArgs::default()
//         .file(
//             "./audio/A Message From Sir David Attenborough A Perfect Planet BBC Earth_320kbps.mp3",
//         )
//         .model("whisper-1")
//         .response_format(AudioResponseFormat::Json)
//         .build()?;

//     let response = client.audio().transcribe(request).await?;
//     println!("{}", response.text);
//     Ok(())
// }

// async fn transcribe_srt() -> Result<(), anyhow::Error> {
//     let client =  util::create_client()?;
//     let request = CreateTranscriptionRequestArgs::default()
//         .file(
//             "./audio/A Message From Sir David Attenborough A Perfect Planet BBC Earth_320kbps.mp3",
//         )
//         .model("whisper-1")
//         .response_format(AudioResponseFormat::Srt)
//         .build()?;

//     let response = client.audio().transcribe_raw(request).await?;
//     println!("{}", String::from_utf8_lossy(response.as_ref()));
//     Ok(())
// }
