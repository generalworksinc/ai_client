use std::error::Error;
use tauri::{CustomMenuItem, Menu, MenuItem, Submenu};
use tauri::{Manager, Window, WindowUrl};
use serde::Deserialize;

use async_openai::{
    config::OpenAIConfig,
    types::{
        AssistantStreamEvent, CreateAssistantRequestArgs, CreateMessageRequest, CreateRunRequest,
        CreateThreadRequest, FunctionObject, MessageDeltaContent, MessageRole, RunObject,
        SubmitToolOutputsRunRequest, ToolsOutputs,
        CreateMessageRequestArgs, CreateRunRequestArgs,
        CreateThreadRequestArgs,
        AssistantToolFileSearchResources, AssistantToolsFileSearch, 
        CreateFileRequest,
        CreateVectorStoreRequest, FilePurpose, MessageAttachment, MessageAttachmentTool,
        MessageContent,  ModifyAssistantRequest, RunStatus,
    },
    Client,
};
use futures::StreamExt;
use crate::API_KEY;


#[tauri::command]
pub async fn assistents_file_search_test(awindow: Window,
    app_handle: tauri::AppHandle,
    params: String,
    timeout_sec: Option<u64>,) -> Result<String, String> {
    #[derive(Deserialize)]
    struct PostData {
        message: Option<String>,
        // data: String,
        id: Option<String>,
    }
    println!("call assistents_stream_test: {:#?}", params);
    let postData = serde_json::from_str::<PostData>(params.as_str()).unwrap();
    match assistant_file_search_example(postData.message.unwrap_or_default().as_str()).await.map_err(|e| format!("{:?}", e)) {
        Ok(_) => {
            Ok("テスト終了".to_string())
        }
        Err(err) => {
            Err(err)
        }
    }
}

async fn assistant_file_search_example(question: &str) -> anyhow::Result<()>{
    //create a client
    let client = Client::with_config(OpenAIConfig::new().with_api_key(API_KEY.read().map(|x| x.clone()).unwrap_or_default()));

    //ask the user for the name of the assistant
    println!("--- Enter the name of your assistant");
    //get user input
    let assistant_name = "example_assistant".to_string();
    // std::io::stdin().read_line(&mut assistant_name).unwrap();

    //ask the user for the instruction set for the assistant
    println!("--- Enter the instruction set for your new assistant");
    //get user input
    // let instructions = "あなたはIQ300の超天才です。どんな問題についても膨大な知識から３つの回答を引き出せます。一つは汎用的かつ最適な回答、１つは超極論、もう一つはその極論の真逆にある超極論です。".to_string();
    let instructions = "あなたは専門の金融アナリストです。ナレッジベースを使用して、監査済み財務諸表に関する質問に答えます。".to_string();
    // std::io::stdin().read_line(&mut instructions).unwrap();

    //create the assistant
    let assistant_request = CreateAssistantRequestArgs::default()
        .name(&assistant_name)
        .instructions(&instructions)
        .model("gpt-4o-mini")
        .tools(vec![
            AssistantToolsFileSearch::default().into(),
        ]).build()?;
    let assistant = client.assistants().create(assistant_request).await?;
    //get the id of the assistant
    let assistant_id = &assistant.id;
    println!("--- 2Enter the instruction set for your new assistant");
    
    //
    // Step 2: Upload files and add them to a Vector Store
    //

    // upload file to add to vector store
    let openai_file = client
        .files()
        .create(CreateFileRequest {
            file: "./input/uber-10k.pdf".into(),
            // file: "./input/a.pdf".into(),
            // file: "./input/doctor.png".into(),
            purpose: FilePurpose::Assistants,
        })
        .await?;
    println!("--- 3Enter the instruction set for your new assistant");

    // Create a vector store called "Financial Statements"
    // add uploaded file to vector store
    let vector_store = client
        .vector_stores()
        .create(CreateVectorStoreRequest {
            name: Some("Financial Statements".into()),
            file_ids: Some(vec![openai_file.id.clone()]),
            ..Default::default()
        })
        .await?;
    println!("--- 4Enter the instruction set for your new assistant");
        //
    // Step 3: Update the assistant to to use the new Vector Store
    //

    let assistant = client
        .assistants()
        .update(
            &assistant.id,
            ModifyAssistantRequest {
                tool_resources: Some(
                    AssistantToolFileSearchResources {
                        vector_store_ids: vec![vector_store.id.clone()],
                    }
                    .into(),
                ),
                ..Default::default()
            },
        )
        .await?;

    println!("--- 5Enter the instruction set for your new assistant");
        //
    // Step 4: Create a thread
    //

    // You can also attach files as Message attachments on your thread. Doing so will create another vector_store associated with the thread, or, if there is already a vector store attached to this thread, attach the new files to the existing thread vector store. When you create a Run on this thread, the file search tool will query both the vector_store from your assistant and the vector_store on the thread.

    // Upload user provided file to OpenAI
    let message_file = client
        .files()
        .create(CreateFileRequest {
            file: "./input/lyft-10k.pdf".into(),
            // file: "./input/cello_hiki_gauche-3955382818.png".into(),
            ..Default::default()
        })
        .await?;

    println!("--- 6Enter the instruction set for your new assistant");
    // Create a thread and attach the file to the message

    let create_message_request = CreateMessageRequestArgs::default()
        .role(MessageRole::User)
        // .content("What was the total annual profit of Uber and Lyft?")
        .content(question)
        .attachments(vec![MessageAttachment {
            file_id: message_file.id.clone(),
            tools: vec![MessageAttachmentTool::FileSearch],
        }])
        .build()?;

    println!("--- 7Enter the instruction set for your new assistant");
    let create_thread_request = CreateThreadRequest {
        messages: Some(vec![create_message_request]),
        ..Default::default()
    };

    let thread = client.threads().create(create_thread_request).await?;
    println!("--- Enter the instruction set for your new assistant");
    //
    // Step 5: Create a run and check the output
    //

    let create_run_request = CreateRunRequest {
        assistant_id: assistant.id.clone(),
        ..Default::default()
    };

    let mut run = client
        .threads()
        .runs(&thread.id)
        .create(create_run_request)
        .await?;

    // poll the status of run until its in a terminal state
    loop {
        //check the status of the run
        match run.status {
            RunStatus::Completed => {
                println!("> Run Completed: {:#?}", run);
                let messages = client
                    .threads()
                    .messages(&thread.id)
                    .list(&[("limit", "10")])
                    .await?;

                for message_obj in messages.data {
                    let message_contents = message_obj.content;
                    for message_content in message_contents {
                        match message_content {
                            MessageContent::Text(text) => {
                                let text_data = text.text;
                                let annotations = text_data.annotations;
                                println!("{}", text_data.value);
                                println!("{annotations:?}");
                            }
                            MessageContent::ImageFile(_) | MessageContent::ImageUrl(_) => {
                                eprintln!("Images not supported on terminal");
                            }
                        }
                    }
                }

                break;
            }
            RunStatus::Failed => {
                println!("> Run Failed: {:#?}", run);
                break;
            }
            RunStatus::Queued => {
                println!("> Run Queued");
            }
            RunStatus::Cancelling => {
                println!("> Run Cancelling");
            }
            RunStatus::Cancelled => {
                println!("> Run Cancelled");
                break;
            }
            RunStatus::Expired => {
                println!("> Run Expired");
                break;
            }
            RunStatus::RequiresAction => {
                println!("> Run Requires Action");
            }
            RunStatus::InProgress => {
                println!("> In Progress ...");
            }
            RunStatus::Incomplete => {
                println!("> Run Incomplete");
            }
        }

        // wait for 1 sec before polling run object again
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        //retrieve the run
        run = client.threads().runs(&thread.id).retrieve(&run.id).await?;
    }

    // clean up
    client.threads().delete(&thread.id).await?;
    client.vector_stores().delete(&vector_store.id).await?;
    client.files().delete(&openai_file.id).await?;
    client.files().delete(&message_file.id).await?;
    client.assistants().delete(&assistant.id).await?;

    Ok(())
}
