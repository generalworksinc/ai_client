use serde::Deserialize;
use tauri::Window;

use crate::API_KEY;
use async_openai::{
    config::OpenAIConfig,
    types::{
        AudioResponseFormat, CreateTranslationRequestArgs,
    },
    Client,
};
use futures::StreamExt;

#[tauri::command]
pub async fn assistants_audio_translate_test(
    awindow: Window,
    app_handle: tauri::AppHandle,
    params: String,
    timeout_sec: Option<u64>,
) -> Result<String, String> {
    #[derive(Deserialize)]
    struct PostData {
        message: Option<String>,
        // data: String,
        id: Option<String>,
    }
    println!("call assistents_stream_test: {:#?}", params);
    let postData = serde_json::from_str::<PostData>(params.as_str()).unwrap();
    match audio_translate_example(postData.message.unwrap_or_default().as_str())
        .await
        .map_err(|e| format!("{:?}", e))
    {
        Ok(_) => Ok("テスト終了".to_string()),
        Err(err) => {
            println!("Error: {:#?}", err);
            Err(err)
        }
    }
}

fn create_client() -> Client<OpenAIConfig> {
    Client::with_config(
        OpenAIConfig::new().with_api_key(API_KEY.read().map(|x| x.clone()).unwrap_or_default()),
    )
}
async fn audio_translate_example(question: &str) -> anyhow::Result<()> {
    translate_verbose_json().await?;
    translate_srt().await?;

    Ok(())
}

async fn translate_srt() -> Result<(), anyhow::Error> {
    let client = create_client();
    let request = CreateTranslationRequestArgs::default()
        .file("./audio/koshish karne walon ki haar nahi hoti by amitabh bachchan_320kbps.mp3")
        .model("whisper-1")
        .response_format(AudioResponseFormat::Srt)
        .build()?;

    let response = client.audio().translate_raw(request).await?;

    println!("translate_srt:");
    println!("{}", String::from_utf8_lossy(response.as_ref()));
    Ok(())
}

async fn translate_verbose_json() -> Result<(), anyhow::Error> {
    let client = create_client();
    // Credits and Source for audio: https://www.youtube.com/watch?v=bHWmzQ4HTS0
    let request = CreateTranslationRequestArgs::default()
        .file("./audio/koshish karne walon ki haar nahi hoti by amitabh bachchan_320kbps.mp3")
        .model("whisper-1")
        .build()?;

    let response = client.audio().translate(request).await?;

    println!("translate_verbose_json:");
    println!("{}", response.text);

    Ok(())
}
