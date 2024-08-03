use std::str::FromStr;
use tauri::{CustomMenuItem, Menu, MenuItem, Submenu};
use tauri::{Manager, Window, WindowUrl};
use serde::Deserialize;


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
pub async fn assistents_test(awindow: Window,
    app_handle: tauri::AppHandle,
    params: String,
    timeout_sec: Option<u64>,) -> Result<String, String> {
    #[derive(Deserialize)]
    struct PostData {
        message: Option<String>,
        // data: String,
        id: Option<String>,
    }
    println!("call assistents_test: {:#?}", params);
    let postData = serde_json::from_str::<PostData>(params.as_str()).unwrap();
    match assistant_example(postData.message.unwrap_or_default().as_str()).await.map_err(|e| format!("{:?}", e)) {
        Ok(_) => {
            Ok("テスト終了".to_string())
        }
        Err(err) => {
            Err(err)
        }
    }
}

async fn assistant_example(question: &str) -> anyhow::Result<()>{
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
    let instructions = "あなたはギャルな魔王です。世界を支配する魔王として、ギャル語で質問に答えてください".to_string();
    // std::io::stdin().read_line(&mut instructions).unwrap();

    //create the assistant
    let assistant_request = CreateAssistantRequestArgs::default()
        .name(&assistant_name)
        .instructions(&instructions)
        .model("gpt-4o-mini")
        .build()?;
    let assistant = client.assistants().create(assistant_request).await?;
    //get the id of the assistant
    let assistant_id = &assistant.id;

    //create a thread for the conversation
    let thread_request = CreateThreadRequestArgs::default().build()?;
    let thread = client.threads().create(thread_request.clone()).await?;

    println!("--- How can I help you?");
    //get user input
    let mut input = question.to_string();

    //create a message for the thread
    let message = CreateMessageRequestArgs::default()
        .role(MessageRole::User)
        .content(input.clone())
        .build()?;

    println!("thread:1 {:#?}", thread);
    //attach message to the thread
    let _message_obj = client
        .threads()
        .messages(&thread.id)
        .create(message)
        .await?;

        println!("thread:2 {:#?}", thread);
    //create a run for the thread
    let run_request = CreateRunRequestArgs::default()
        .assistant_id(assistant_id)
        .build()?;
    println!("thread:3 {:#?}", thread);
    let run = client
        .threads()
        .runs(&thread.id)
        .create(run_request)
        .await?;

    println!("thread:4 {:#?}", thread);
    //wait for the run to complete
    let mut awaiting_response = true;
    while awaiting_response {
        //retrieve the run
        println!("thread:5...{:#?}", thread);
        let run = client.threads().runs(&thread.id).retrieve(&run.id).await?;
        //check the status of the run
        match run.status {
            RunStatus::Completed => {
                awaiting_response = false;
                //retrieve the response from the run
                let query = [("limit", "1")]; //limit the list responses to 1 message
                let response = client.threads().messages(&thread.id).list(&query).await?;
                //get the message id from the response
                let message_id = response.data.first().unwrap().id.clone();
                //get the message from the response
                let message = client
                    .threads()
                    .messages(&thread.id)
                    .retrieve(&message_id)
                    .await?;
                //get the content from the message
                let content = message.content.first().unwrap();
                //get the text from the content
                let text = match content {
                    MessageContent::Text(text) => text.text.value.clone(),
                    MessageContent::ImageFile(_) | MessageContent::ImageUrl(_) => {
                        panic!("imaged are not expected in this example");
                    }
                };
                //print the text
                println!("--- Response: {}\n", text);
            }
            RunStatus::Failed => {
                awaiting_response = false;
                println!("--- Run Failed: {:#?}", run);
            }
            RunStatus::Queued => {
                println!("--- Run Queued");
            }
            RunStatus::Cancelling => {
                println!("--- Run Cancelling");
            }
            RunStatus::Cancelled => {
                println!("--- Run Cancelled");
            }
            RunStatus::Expired => {
                println!("--- Run Expired");
            }
            RunStatus::RequiresAction => {
                println!("--- Run Requires Action");
            }
            RunStatus::InProgress => {
                println!("--- In Progress ...");
            }
            RunStatus::Incomplete => {
                println!("--- Run Incomplete");
            }
        }
        //wait for 1 second before checking the status again
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    //once we have broken from the main loop we can delete the assistant and thread
    println!("assistant_id: {:?}", assistant_id);
    println!("thread_id: {:?}", thread.id);
    client.assistants().delete(assistant_id).await?;
    client.threads().delete(&thread.id).await?;
    client.threads().delete("thread_FUX4wgtpMMxpOQjN2wZ2lYhp").await?;
    Ok(())
}