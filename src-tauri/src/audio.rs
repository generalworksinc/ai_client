use crate::constants::OPENAI_MAXIMUM_CONTENT_SIZE_BYTES;
use crate::util;
use anyhow::Context;
use core::str;
use serde_json::json;
use std::process::Command;
use std::str::FromStr;
use tauri::{Emitter, Window};
use tempfile::tempdir;

use async_openai::types::{
    AudioInput, AudioResponseFormat, CreateTranscriptionRequestArgs, TimestampGranularity,
};

#[tauri::command]
pub async fn audio_transcribe(
    window: Window,
    app_handle: tauri::AppHandle,
    filepath: String,
) -> Result<String, String> {
    println!("call audio_transcribe: {:#?}", filepath);
    match audio_transcribe_exec(filepath.as_str())
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

async fn audio_transcribe_exec(file_path: &str) -> anyhow::Result<serde_json::Value> {
    // transcribe_json().await?;
    transcribe_verbose_json(file_path).await
}

async fn transcribe_verbose_json(file_path_str: &str) -> Result<serde_json::Value, anyhow::Error> {
    //filepathから、ファイル名をbinaryを取得
    let file_path = std::path::PathBuf::from_str(file_path_str)?;
    let file_name = file_path
        .file_name()
        .context("Invalid file path")?
        .to_string_lossy();
    let file_binary = util::get_file_binary(file_path.as_path())?;
    let file_byte_size = file_binary.len();

    println!("file_binary len: {:?}", file_binary.len());

    //ファイルをサイズから分割して複数にしてアップする
    let output = Command::new("ffprobe")
        .arg("-v")
        .arg("error")
        .arg("-show_entries")
        .arg("format=duration")
        .arg("-of")
        .arg("csv=p=0")
        .arg(format!("{}", file_path.to_string_lossy()).as_str())
        .output()?;

    // コマンドの出力をUTF-8として解析する
    let duration_str = str::from_utf8(&output.stdout)?;

    // 取得したdurationを表示する
    let duration = f64::from_str(duration_str.trim())?; // 前後の空白を取り除く
    println!("Duration: {}", duration);

    let client = util::create_client()?;
    let mut text_full = "".to_string();

    if OPENAI_MAXIMUM_CONTENT_SIZE_BYTES < file_byte_size as u64 {
        let split_count = (file_byte_size as u64 / OPENAI_MAXIMUM_CONTENT_SIZE_BYTES) as u64 + 1;
        let split_time = duration / split_count as f64;

        // 一時的なディレクトリを作成
        let temp_dir = tempdir()?;
        println!("Temporary directory created at: {:?}", temp_dir.path());

        let split_output = Command::new("ffmpeg")
            .arg("-i")
            .arg(file_path.as_os_str().to_string_lossy().as_ref())
            .arg("-f")
            .arg("segment")
            .arg("-segment_time")
            .arg(split_time.to_string())
            .arg("-c")
            .arg("copy")
            .arg(temp_dir.path().join("output%03d.mp3"))
            .output()?;

        let split_result = str::from_utf8(&split_output.stdout)?;
        println!("split_result: {}", split_result);

        if let Ok(read_dir) = temp_dir.path().read_dir() {
            for entry in read_dir.filter_map(|x| x.ok()) {
                let file_path_buff = entry.path().clone();
                let file_name = file_path_buff.file_name().unwrap().to_string_lossy();
                let binary = util::get_file_binary(file_path_buff.as_path())?;
                println!("file_len: {file_name:?}, {:?}", binary.len());
            }
        }
        if let Ok(read_dir) = temp_dir.path().read_dir() {
            for entry in read_dir.filter_map(|x| x.ok()) {
                let file_path_buff = entry.path().clone();
                let file_name = file_path_buff.file_name().unwrap().to_string_lossy();
                let binary = util::get_file_binary(file_path_buff.as_path())?;
                let audio_input = AudioInput::from_vec_u8(file_name.to_string(), binary);

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
                text_full.push_str(response.text.as_str());
            }
        }
    } else {
        let audio_input = AudioInput::from_vec_u8(file_name.to_string(), file_binary);

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
    }

    Ok(json!({
        "text": text_full,
        // "words": response.words,
        // "segments": response.segments,
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
