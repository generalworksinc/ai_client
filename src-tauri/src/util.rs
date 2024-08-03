use std::error::Error;
use tauri::{CustomMenuItem, Menu, MenuItem, Submenu};
use tauri::{Manager, Window, WindowUrl};
use serde::Deserialize;
use anyhow::{Context, Result};

use async_openai::{
    config::OpenAIConfig,
    types::{
        CreateSpeechRequestArgs, SpeechModel, Voice,
        AudioResponseFormat, CreateTranscriptionRequestArgs, TimestampGranularity,
        CreateTranslationRequestArgs,
        AssistantStreamEvent, CreateAssistantRequestArgs, CreateMessageRequest, CreateRunRequest,
        CreateThreadRequest, FunctionObject, MessageDeltaContent, MessageRole, RunObject,
        SubmitToolOutputsRunRequest, ToolsOutputs,
        CreateMessageRequestArgs, CreateRunRequestArgs,
        CreateThreadRequestArgs,
        AssistantToolFileSearchResources, AssistantToolsFileSearch, 
        CreateFileRequest,
        CreateVectorStoreRequest, FilePurpose, MessageAttachment, MessageAttachmentTool,
        MessageContent,  ModifyAssistantRequest, RunStatus,
        AssistantToolCodeInterpreterResources, AssistantTools, MessageContentTextAnnotations, 
    },
    Client,
};
use futures::StreamExt;
use crate::API_KEY;

pub fn create_client() -> Result<Client<OpenAIConfig>> {
    let api_key = API_KEY.read().map_err(|e| anyhow::anyhow!("failed to read API_KEY: {:?}", e))?;
    Ok(Client::with_config(OpenAIConfig::new().with_api_key(api_key.clone())))
}