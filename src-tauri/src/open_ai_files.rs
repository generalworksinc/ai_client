use crate::models::chat::ChatApiMessage;
use crate::util::{self, create_client};
use crate::constants::{DIR_ASSISTANTS, DIR_OPEN_AI_FILES, DIR_OPEN_AI_VECTORS, DIR_THREADS};
use crate::SAVING_DIRECTORY;
use crate::models::open_ai::{OpenAIFileData, OpenAIVectorData};
use base64::prelude::*;
use futures::StreamExt;
use serde::Deserialize;
use chrono::{TimeZone, Utc, Local};
use serde_json::Value;
use std::fs::File;
use std::io::prelude::*;
use tauri::Window;

use async_openai::{
    config::OpenAIConfig,
    types::{
        self, AssistantStreamEvent, CreateAssistantRequestArgs, CreateFileRequestArgs, CreateVectorStoreRequest,
        CreateImageRequestArgs, CreateMessageRequestArgs, CreateMessageRequestContent,
        CreateRunRequestArgs, CreateThreadRequestArgs, FileInput, ImageFile, ImageInput, ImageUrl,
        MessageContentInput, MessageDeltaContent, MessageRequestContentTextObject, MessageRole,
        RunObject, SubmitToolOutputsRunRequest, ToolsOutputs, OpenAIFile
    },
    Client,
};


#[tauri::command]
pub async fn reflesh_vectors(app_handle: tauri::AppHandle) -> Result<String, String> {
    
    let dir = unsafe { SAVING_DIRECTORY.clone() };
    let vector_path = std::path::Path::new(dir.as_str()).join(DIR_OPEN_AI_VECTORS);
    if vector_path.exists() {
        if let Ok(read_dir) = vector_path.read_dir() {
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

                        let vector_file_string = std::fs::read_to_string(entry.path()).map_err(|x| x.to_string()).unwrap();
                        let mut vectorData: OpenAIVectorData = if vector_file_string.is_empty() { OpenAIVectorData::default() } else { serde_json::from_str(vector_file_string.as_str()).unwrap()};
                        
                        if vectorData.created.is_none() {
                            vectorData.time = Some(datetime.format("%Y-%m-%d %H:%M:%S").to_string());
                        } else {
                            chrono::Utc.timestamp_millis_opt(vectorData.created.unwrap()*1000).map(|x| {
                                vectorData.time = Some(x.format("%Y-%m-%d %H:%M:%S").to_string());
                            });
                        }

                        if vectorData.id.is_none() {
                            vectorData.id = Some(entry.file_name().to_string_lossy().to_string());
                        }
                        Some(vectorData)
                    } else {
                        None
                    }
                })
                .collect::<Vec<OpenAIVectorData>>();
            return Ok(serde_json::to_string(&data_vec).unwrap());
        }
    }
    Err("".to_string())
}

#[tauri::command]
pub async fn reflesh_openai_files(app_handle: tauri::AppHandle) -> Result<String, String> {
    let client = create_client().map_err(|err| err.to_string())?;
    // let response = client.files().list("").await.map_err(|err| err.to_string())?;
    // response.data.into_iter().for_each(|x| {
    //     println!("x.filename: {:?}", x.filename);
    // });

    let dir = unsafe { SAVING_DIRECTORY.clone() };
    let openai_file_path = std::path::Path::new(dir.as_str()).join(DIR_OPEN_AI_FILES);
    if openai_file_path.exists() {
        if let Ok(read_dir) = openai_file_path.read_dir() {
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

                        let openai_file_string = std::fs::read_to_string(entry.path()).map_err(|x| x.to_string()).unwrap();
                        let mut openAIFileData: OpenAIFileData = if openai_file_string.is_empty() { OpenAIFileData::default() } else { serde_json::from_str(openai_file_string.as_str()).unwrap()};
                        openAIFileData.time = Some(datetime.format("%Y-%m-%d %H:%M:%S").to_string());
                        
                        if openAIFileData.id.is_none() {
                            openAIFileData.id = Some(entry.file_name().to_string_lossy().to_string());
                        }
                        Some(openAIFileData)
                    } else {
                        None
                    }
                })
                .collect::<Vec<OpenAIFileData>>();
            return Ok(serde_json::to_string(&data_vec).unwrap());
        }
    }
    Err("".to_string())
}


#[tauri::command]
pub async fn make_vector(
    window: Window,
    app_handle: tauri::AppHandle,
    params: String,
    timeout_sec: Option<u64>,
) -> Result<String, String> {
    println!("make_vector");
    #[derive(Deserialize)]
    struct PostData {
        // message: Option<String>,
        // assistant_name: String,
        // instructions: Option<String>,
        vector_name: String,
        open_ai_file_id_list: Vec<String>,
    }
    // println!("call assistents_test: {:#?}", params);
    let post_data = serde_json::from_str::<PostData>(params.as_str()).unwrap();
    
    
    match exec_make_vector(
        // &post_data.assistant_name,
        // // postData.message.unwrap_or_default().as_str(),
        // post_data.instructions.unwrap_or_default().as_str(),
        post_data.vector_name.as_str(),
        post_data.open_ai_file_id_list,
    )
    .await
    .map_err(|e| format!("{:?}", e))
    {
        Ok(_) => Ok("テスト終了".to_string()),
        Err(err) => Err(err),
    }
}

async fn exec_make_vector(vector_name: &str, open_ai_file_id_list:  Vec<String>) -> anyhow::Result<()> {
    //create a client
    let client = create_client()?;
    println!("open_ai_file_id_list: {:#?}", open_ai_file_id_list);
    //make Vector
    let vector_store = client
        .vector_stores()
        .create(CreateVectorStoreRequest {
            name: Some(vector_name.into()),
            file_ids: Some(open_ai_file_id_list),
            ..Default::default()
        })
        .await?;
    

    //作成したvector情報をローカルに保存する
    let dir = unsafe { SAVING_DIRECTORY.clone() };
    let files_dir_path = std::path::Path::new(dir.as_str()).join(DIR_OPEN_AI_VECTORS);

    if !files_dir_path.exists() {
        std::fs::create_dir_all(files_dir_path.as_path())?;
    }

    let file_path = files_dir_path.join(vector_store.id.clone());
    let mut f = File::create(file_path).unwrap();
    let json_data = serde_json::to_string(&vector_store)?;
    f.write_all(json_data.as_bytes())?;

    // client.vector_stores().delete(&vector_store.id).await?;
    Ok(())
}

#[tauri::command]
pub async fn upload_files(
    window: Window,
    app_handle: tauri::AppHandle,
    params: String,
    timeout_sec: Option<u64>,
) -> Result<String, String> {

    #[derive(Deserialize)]
    struct PostData {
        // message: Option<String>,
        // assistant_name: String,
        // instructions: Option<String>,
        file_list: Option<Vec<Vec<String>>>,
    }
    // println!("call assistents_test: {:#?}", params);
    let post_data = serde_json::from_str::<PostData>(params.as_str()).unwrap();
    
    
    match exec_upload_files(
        // &post_data.assistant_name,
        // // postData.message.unwrap_or_default().as_str(),
        // post_data.instructions.unwrap_or_default().as_str(),
        post_data.file_list,
    )
    .await
    .map_err(|e| format!("{:?}", e))
    {
        Ok(_) => Ok("テスト終了".to_string()),
        Err(err) => Err(err),
    }
}

async fn exec_upload_files(file_list: Option<Vec<Vec<String>>>) -> anyhow::Result<()> {
    //create a client
    let client = create_client()?;

    //ファイルがある場合は、ファイルをアップロードする
    let mut file_id_list: Vec<OpenAIFile> = vec![];
    if let Some(file_list) = file_list {
        for file in file_list {
            let file_name = file[1].clone();
            let file_body = file[0].clone();
            let file_binary: Vec<u8>;
            //////////////////////////////////////////////////////////////////////////////////////////
            if let Some((file_type, file_body)) = file_body.split_once("base64,") {
                println!("file_type: {:?}", file_type);
                file_binary = BASE64_STANDARD.decode(file_body)?;
            } else {
                return Err(anyhow::anyhow!("Invalid file format"));
            }
            println!("file_binary len: {:?}", file_binary.len());

            let bytes = bytes::Bytes::from(file_binary);
            let file_input = FileInput::from_bytes(file_name.to_string(), bytes);

            let create_file_request = types::CreateFileRequestArgs::default()
                .file(file_input)
                .build()?;
            let create_file = client.files().create(create_file_request).await?;
            file_id_list.push(create_file);
        }
    }

    //アップしたファイル情報をローカルに保存する
    let dir = unsafe { SAVING_DIRECTORY.clone() };
    let files_dir_path = std::path::Path::new(dir.as_str()).join(DIR_OPEN_AI_FILES);

    if !files_dir_path.exists() {
        std::fs::create_dir_all(files_dir_path.as_path())?;
    }
    for open_ai_file in file_id_list {
        let file_path = files_dir_path.join(open_ai_file.id.clone());
        let mut f = File::create(file_path).unwrap();
        let json_data = serde_json::to_string(&open_ai_file)?;
        f.write_all(json_data.as_bytes())?;

        // println!("delete file: {:?}", open_ai_file.id);
        // client.files().delete(open_ai_file.id.as_str()).await?;
    }
    // client.assistants().delete(&assistant_object.id).await?;
    Ok(())
}

#[tauri::command]
pub async fn delete_vector(app_handle: tauri::AppHandle, id: String) -> Result<String, String> {
    //OpenAIFileを削除
    println!("vector_id: {:#?}", id);
    let client = create_client().map_err(|err| err.to_string())?;
    match client
        .vector_stores()
        .delete(id.as_str())
        .await
        .map_err(|x| x.to_string())
    {
        Ok(_) => {
            println!("vector_store delete success {:?}", id);
        }
        Err(err) => {
            eprintln!("vector_store delete error: {:#?}", err);
        }
    }

    let dir = unsafe { SAVING_DIRECTORY.clone() };

    //open_ai_fileファイルを削除
    let file_path_open_ai_file = std::path::Path::new(dir.as_str())
        .join(DIR_OPEN_AI_VECTORS)
        .join(id.clone());
    if file_path_open_ai_file.exists() {
        //削除
        std::fs::remove_file(file_path_open_ai_file).map_err(|x| x.to_string())?;
    }
    
    Ok("ファイルID削除しました".to_string())
}

#[tauri::command]
pub async fn delete_openai_file(app_handle: tauri::AppHandle, id: String) -> Result<String, String> {
    //OpenAIFileを削除
    println!("openai_file_id: {:#?}", id);
    let client = create_client().map_err(|err| err.to_string())?;
    match client
        .files()
        .delete(id.as_str())
        .await
        .map_err(|x| x.to_string())
    {
        Ok(_) => {
            println!("open_ai_files delete success {:?}", id);
        }
        Err(err) => {
            eprintln!("open_ai_files delete error: {:#?}", err);
        }
    }

    let dir = unsafe { SAVING_DIRECTORY.clone() };

    //open_ai_fileファイルを削除
    let file_path_open_ai_file = std::path::Path::new(dir.as_str())
        .join(DIR_OPEN_AI_FILES)
        .join(id.clone());
    if file_path_open_ai_file.exists() {
        //削除
        std::fs::remove_file(file_path_open_ai_file).map_err(|x| x.to_string())?;
    }
    
    Ok("ファイルID削除しました".to_string())
}
