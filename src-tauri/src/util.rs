use anyhow::{Context, Result};
use serde::Deserialize;
use std::error::Error;
use tauri::{CustomMenuItem, Menu, MenuItem, Submenu};
use tauri::{Manager, Window, WindowUrl};

use crate::API_KEY;
use async_openai::{
    config::OpenAIConfig,
    types::{
        AssistantStreamEvent, AssistantToolCodeInterpreterResources,
        AssistantToolFileSearchResources, AssistantTools, AssistantToolsFileSearch,
        AudioResponseFormat, CreateAssistantRequestArgs, CreateFileRequest, CreateMessageRequest,
        CreateMessageRequestArgs, CreateRunRequest, CreateRunRequestArgs, CreateSpeechRequestArgs,
        CreateThreadRequest, CreateThreadRequestArgs, CreateTranscriptionRequestArgs,
        CreateTranslationRequestArgs, CreateVectorStoreRequest, FilePurpose, FunctionObject,
        MessageAttachment, MessageAttachmentTool, MessageContent, MessageContentTextAnnotations,
        MessageDeltaContent, MessageRole, ModifyAssistantRequest, RunObject, RunStatus,
        SpeechModel, SubmitToolOutputsRunRequest, TimestampGranularity, ToolsOutputs, Voice,
    },
    Client,
};
use futures::StreamExt;

pub fn create_client() -> Result<Client<OpenAIConfig>> {
    let api_key = API_KEY
        .read()
        .map_err(|e| anyhow::anyhow!("failed to read API_KEY: {:?}", e))?;
    Ok(Client::with_config(
        OpenAIConfig::new().with_api_key(api_key.clone()),
    ))
}
