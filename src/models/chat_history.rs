use std::fmt;

use async_openai::types::{ChatCompletionRequestAssistantMessage, ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage, ChatCompletionRequestUserMessage};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatRole {
    System,
    User,
    Assistant
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
}

#[derive(Debug, Clone, Default)]
pub struct ChatHistory {
    pub messages: Vec<ChatMessage>,
}

impl Into<Vec<ChatCompletionRequestMessage>> for ChatHistory {
    fn into(self) -> Vec<ChatCompletionRequestMessage> {
        self.messages.into_iter().map(|msg| match msg.role {
            ChatRole::System => ChatCompletionRequestMessage::System(
                ChatCompletionRequestSystemMessage {
                    content: async_openai::types::ChatCompletionRequestSystemMessageContent::Text(msg.content),
                    ..Default::default()
                }
            ),
            ChatRole::User => ChatCompletionRequestMessage::User(
                ChatCompletionRequestUserMessage {
                    content: async_openai::types::ChatCompletionRequestUserMessageContent::Text(msg.content),
                    ..Default::default()
                }
            ),
            ChatRole::Assistant => ChatCompletionRequestMessage::Assistant(
                ChatCompletionRequestAssistantMessage {
                    content: Some(async_openai::types::ChatCompletionRequestAssistantMessageContent::Text(msg.content)),
                    ..Default::default()
                }
            ),
        }).collect()
    }
}

impl ChatHistory {

    pub fn set_system_message(&mut self, content: String) {
        self.messages.insert(0, ChatMessage { role: ChatRole::System, content: content });
    }

    pub fn add_message(&mut self, role: ChatRole, content: String) {
        self.messages.push(ChatMessage { role, content });
    }

    pub fn add_user_message(&mut self, content: String) {
        self.add_message(ChatRole::User, content);
    }

    pub fn add_assistant_message(&mut self, content: String) {
        self.add_message(ChatRole::Assistant, content);
    }

    pub fn clear(&mut self) {
        self.messages.clear();
    }

    pub fn last_message(&self) -> Option<&ChatMessage> {
        self.messages.last()
    }
}

impl fmt::Display for ChatHistory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for message in &self.messages {
            let role = match message.role {
                ChatRole::System => "System",
                ChatRole::User => "User",
                ChatRole::Assistant => "Assistant",
            };
            writeln!(f, "Role: {}: Content: {}", role, message.content)?;
        }
        Ok(())
    }
}