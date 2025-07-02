use async_openai::types::{ChatCompletionRequestAssistantMessage, ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage, ChatCompletionRequestUserMessage};



#[derive(Debug, Clone)]
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
    fn add_message(&mut self, role: ChatRole, content: String) {
        self.messages.push(ChatMessage { role, content });
    }

    pub fn add_system_message(&mut self, content: String) {
        self.add_message(ChatRole::System, content);
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