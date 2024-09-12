use serde::Deserialize;
use std::error::Error;
use tauri::Window;

use crate::API_KEY;
use async_openai::{
    config::OpenAIConfig,
    types::{
        AssistantStreamEvent, CreateAssistantRequestArgs, CreateMessageRequestArgs,
        CreateRunRequestArgs, CreateThreadRequestArgs, FunctionObject, MessageDeltaContent,
        MessageRole, RunObject, SubmitToolOutputsRunRequest, ToolsOutputs,
    },
    Client,
};
use futures::StreamExt;

#[tauri::command]
pub async fn assistents_stream_test(
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
    match assistant_stream_example(postData.message.unwrap_or_default().as_str())
        .await
        .map_err(|e| format!("{:?}", e))
    {
        Ok(_) => Ok("テスト終了".to_string()),
        Err(err) => Err(err),
    }
}

async fn assistant_stream_example(question: &str) -> anyhow::Result<()> {
    //create a client
    let client = Client::with_config(
        OpenAIConfig::new().with_api_key(API_KEY.read().map(|x| x.clone()).unwrap_or_default()),
    );

    //ask the user for the name of the assistant
    println!("--- Enter the name of your assistant");
    //get user input
    let assistant_name = "example_assistant".to_string();
    // std::io::stdin().read_line(&mut assistant_name).unwrap();

    //ask the user for the instruction set for the assistant
    println!("--- Enter the instruction set for your new assistant");
    //get user input
    // let instructions = "あなたはIQ300の超天才です。どんな問題についても膨大な知識から３つの回答を引き出せます。一つは汎用的かつ最適な回答、１つは超極論、もう一つはその極論の真逆にある超極論です。".to_string();
    let instructions =
        "あなたは天気botです。用意されている関数を使用して質問に答えます。".to_string();
    // std::io::stdin().read_line(&mut instructions).unwrap();

    //create the assistant
    let assistant_request = CreateAssistantRequestArgs::default()
        .name(&assistant_name)
        .instructions(&instructions)
        .model("gpt-4o-mini")
        .tools(vec![
            FunctionObject {
                strict: Some(false),
                name: "get_current_temperature".into(),
                description: Some("Get the current temperature for a specific location".into()),
                parameters: Some(serde_json::json!(
                    {
                        "type": "object",
                        "properties": {
                        "location": {
                            "type": "string",
                            "description": "The city and state, e.g., San Francisco, CA"
                        },
                        "unit": {
                            "type": "string",
                            "enum": ["Celsius", "Fahrenheit"],
                            "description": "The temperature unit to use. Infer this from the user's location."
                        }
                        },
                        "required": ["location", "unit"]
                    }
                ))
            }.into(),

            FunctionObject {
                strict: Some(false),
                name: "get_rain_probability".into(),
                description: Some("Get the probability of rain for a specific location".into()),
                parameters: Some(serde_json::json!(
                    {
                        "type": "object",
                        "properties": {
                        "location": {
                            "type": "string",
                            "description": "The city and state, e.g., San Francisco, CA"
                        }
                        },
                        "required": ["location"]
                    }
                ))
            }.into()
        ]).build()?;
    let assistant = client.assistants().create(assistant_request).await?;
    //get the id of the assistant
    let assistant_id = &assistant.id;

    // Step 2: Create a Thread and add Messages

    //create a thread for the conversation
    let thread_request = CreateThreadRequestArgs::default().build()?;
    let thread = client.threads().create(thread_request.clone()).await?;

    println!("--- How can I help you?");
    //get user input
    let input = question.to_string();

    //create a message for the thread
    let message = CreateMessageRequestArgs::default()
        .role(MessageRole::User)
        .content(input.clone())
        .build()?;

    //attach message to the thread
    let _message_obj = client
        .threads()
        .messages(&thread.id)
        .create(message)
        .await?;

    // Step 3: Initiate a Run

    //create a run for the thread
    let run_request = CreateRunRequestArgs::default()
        .assistant_id(assistant_id)
        .stream(true)
        .build()?;

    let mut event_stream = client
        .threads()
        .runs(&thread.id)
        .create_stream(run_request.clone())
        .await?;

    // let mut event_stream = client
    //     .threads()
    //     .runs(&thread.id)
    //     .create_stream(CreateRunRequest {
    //         assistant_id: assistant.id.clone(),
    //         stream: Some(true),
    //         ..Default::default()
    //     })
    //     .await?;

    let mut task_handle = None;

    while let Some(event) = event_stream.next().await {
        match event {
            Ok(event) => match event {
                AssistantStreamEvent::ThreadRunRequiresAction(run_object) => {
                    println!("thread.run.requires_action: run_id:{}", run_object.id);
                    let client = client.clone();
                    task_handle = Some(tokio::spawn(async move {
                        handle_requires_action(client, run_object).await
                    }));
                }
                AssistantStreamEvent::ThreadMessageDelta(delta) => {
                    if let Some(contents) = delta.delta.content {
                        for content in contents {
                            // only text is expected here and no images
                            if let MessageDeltaContent::Text(text) = content {
                                if let Some(text) = text.text {
                                    if let Some(text) = text.value {
                                        print!("{}", text);
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {
                    // println!("\nEvent: {event:?}\n")
                }
            },
            Err(e) => {
                eprintln!("Error: {e}");
            }
        }
    }

    // wait for task to handle required action and submit tool outputs
    if let Some(task_handle) = task_handle {
        let _ = tokio::join!(task_handle);
    }

    //once we have broken from the main loop we can delete the assistant and thread
    println!("assistant_id: {:?}", assistant_id);
    println!("thread_id: {:?}", thread.id);
    client.assistants().delete(assistant_id).await?;
    client.threads().delete(&thread.id).await?;
    client
        .threads()
        .delete("thread_FUX4wgtpMMxpOQjN2wZ2lYhp")
        .await?;
    Ok(())
}

async fn handle_requires_action(client: Client<OpenAIConfig>, run_object: RunObject) {
    let mut tool_outputs: Vec<ToolsOutputs> = vec![];
    if let Some(ref required_action) = run_object.required_action {
        for tool in &required_action.submit_tool_outputs.tool_calls {
            if tool.function.name == "get_current_temperature" {
                tool_outputs.push(ToolsOutputs {
                    tool_call_id: Some(tool.id.clone()),
                    output: Some("57".into()),
                })
            }

            if tool.function.name == "get_rain_probability" {
                tool_outputs.push(ToolsOutputs {
                    tool_call_id: Some(tool.id.clone()),
                    output: Some("0.06".into()),
                })
            }
        }

        if let Err(e) = submit_tool_outputs(client, run_object, tool_outputs).await {
            eprintln!("Error on submitting tool outputs: {e}");
        }
    }
}

async fn submit_tool_outputs(
    client: Client<OpenAIConfig>,
    run_object: RunObject,
    tool_outputs: Vec<ToolsOutputs>,
) -> Result<(), Box<dyn Error>> {
    let mut event_stream = client
        .threads()
        .runs(&run_object.thread_id)
        .submit_tool_outputs_stream(
            &run_object.id,
            SubmitToolOutputsRunRequest {
                tool_outputs,
                stream: Some(true),
            },
        )
        .await?;

    while let Some(event) = event_stream.next().await {
        match event {
            Ok(event) => {
                if let AssistantStreamEvent::ThreadMessageDelta(delta) = event {
                    if let Some(contents) = delta.delta.content {
                        for content in contents {
                            // only text is expected here and no images
                            if let MessageDeltaContent::Text(text) = content {
                                if let Some(text) = text.text {
                                    if let Some(text) = text.value {
                                        print!("{}", text);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Error: {e}");
            }
        }
    }

    Ok(())
}
