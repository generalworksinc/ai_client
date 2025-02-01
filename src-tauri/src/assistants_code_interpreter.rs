use serde::Deserialize;
use tauri::{Emitter, Window};

use crate::API_KEY;
use async_openai::{
    config::OpenAIConfig,
    types::{
        AssistantToolCodeInterpreterResources, AssistantTools, CreateAssistantRequestArgs,
        CreateFileRequest, CreateMessageRequestArgs, CreateRunRequest, CreateThreadRequest,
        FilePurpose, MessageContent, MessageContentTextAnnotations, MessageRole, RunStatus,
    },
    Client,
};
use futures::StreamExt;

#[tauri::command]
pub async fn assistents_code_interpreter_test(
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
    match assistant_code_interpreter_example(postData.message.unwrap_or_default().as_str())
        .await
        .map_err(|e| format!("{:?}", e))
    {
        Ok(_) => Ok("テスト終了".to_string()),
        Err(err) => Err(err),
    }
}

async fn assistant_code_interpreter_example(question: &str) -> anyhow::Result<()> {
    //create a client
    let client = Client::with_config(
        OpenAIConfig::new().with_api_key(API_KEY.read().map(|x| x.clone()).unwrap_or_default()),
    );

    // Upload data file with "assistants" purpose
    let data_file = client
        .files()
        .create(CreateFileRequest {
            file: "./input/CASTHPI.csv".into(),
            purpose: FilePurpose::Assistants,
        })
        .await?;

    //ask the user for the name of the assistant
    println!("--- Enter the name of your assistant");
    //get user input
    let assistant_name = "example_assistant".to_string();
    // std::io::stdin().read_line(&mut assistant_name).unwrap();

    //ask the user for the instruction set for the assistant
    println!("--- Enter the instruction set for your new assistant");
    //get user input
    // let instructions = "あなたはIQ300の超天才です。どんな問題についても膨大な知識から３つの回答を引き出せます。一つは汎用的かつ最適な回答、１つは超極論、もう一つはその極論の真逆にある超極論です。".to_string();
    let instructions = "あなたはデータ処理者です。ファイル内のデータについて質問された場合は、質問に答えるコードを記述して実行します。".to_string();
    // std::io::stdin().read_line(&mut instructions).unwrap();

    //create the assistant
    let assistant_request = CreateAssistantRequestArgs::default()
        .name(&assistant_name)
        .instructions(&instructions)
        .model("gpt-4o-mini")
        .tools(vec![AssistantTools::CodeInterpreter])
        .tool_resources(AssistantToolCodeInterpreterResources {
            file_ids: vec![data_file.id.clone()],
        })
        .build()?;
    let assistant = client.assistants().create(assistant_request).await?;
    //get the id of the assistant
    let assistant_id = &assistant.id;
    println!("--- 2Enter the instruction set for your new assistant");

    let create_message_request = CreateMessageRequestArgs::default()
        .role(MessageRole::User)
        .content("価格指数と年のグラフをpng形式で生成してください")
        // .content(question)
        // .attachments(vec![MessageAttachment {
        //     file_id: message_file.id.clone(),
        //     tools: vec![MessageAttachmentTool::FileSearch],
        // }])
        .build()?;

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

    let mut generated_file_ids: Vec<String> = vec![];

    // poll the status of run until its in a terminal state
    loop {
        //check the status of the run
        match run.status {
            RunStatus::Completed => {
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
                                println!("てきすと：{}", text_data.value);
                                for annotation in annotations {
                                    match annotation {
                                        MessageContentTextAnnotations::FileCitation(object) => {
                                            println!("annotation: file citation : {object:?}");
                                        }
                                        MessageContentTextAnnotations::FilePath(object) => {
                                            println!("annotation: file path: {object:?}");
                                            generated_file_ids.push(object.file_path.file_id);
                                        }
                                    }
                                }
                            }
                            MessageContent::ImageFile(object) => {
                                let file_id = object.image_file.file_id;
                                println!("Retrieving image file_id: {}", file_id);
                                let contents = client.files().content(&file_id).await?;
                                let path = "./output/price_index_vs_year_graph.png";
                                tokio::fs::write(path, contents).await?;
                                print!("Graph file: {path}");
                                generated_file_ids.push(file_id);
                            }
                            MessageContent::ImageUrl(object) => {
                                eprintln!("Got Image URL instead: {object:?}");
                            }
                            MessageContent::Refusal(_) => {
                                eprintln!("Refusals not supported on terminal");
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
    client.files().delete(&data_file.id).await?;
    for file_id in generated_file_ids {
        client.files().delete(&file_id).await?;
    }
    client.assistants().delete(&assistant.id).await?;

    Ok(())
}
