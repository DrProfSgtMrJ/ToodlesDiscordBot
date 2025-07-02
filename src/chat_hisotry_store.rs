use async_trait::async_trait;
use std::collections::HashMap;
use tokio::sync::RwLock;
use std::sync::Arc;


use crate::models::{ChatHistory, ChatMessage};

#[async_trait]
pub trait ChatHistoryStore {
    async fn add_chat_message(&self, user_id: &str, message: ChatMessage);
    async fn get_chat_history(&self, user_id: &str) -> Option<ChatHistory>;
}

pub struct InMemoryChatHistoryStore {
    store: Arc<RwLock<HashMap<String, ChatHistory>>>,
}

impl InMemoryChatHistoryStore {
    pub fn new() -> Self {
        InMemoryChatHistoryStore {
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl ChatHistoryStore for InMemoryChatHistoryStore {

    async fn add_chat_message(&self, user_id: &str, message: ChatMessage) {
        let mut store = self.store.write().await;
        let history = store.entry(user_id.to_string()).or_insert_with(ChatHistory::default);
        history.add_message(message.role.clone(), message.content);
    }

    async fn get_chat_history(&self, user_id: &str) -> Option<ChatHistory> {
        let store = self.store.read().await;
        store.get(user_id).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ChatMessage, ChatRole};

    #[tokio::test]
    async fn test_in_memory_chat_history_store() {
        let store = InMemoryChatHistoryStore::new();
        let user_id = "test_user";

        // Check that user_id isn't in store initially
        assert!(store.get_chat_history(user_id).await.is_none());

        // Create a message and add it to the store
        let message = ChatMessage {
            role: ChatRole::User,
            content: "Hello, Toodles!".to_string(),
        };
        store.add_chat_message(user_id, message).await;

        // Retrieve the chat history for the user
        let history = store.get_chat_history(user_id).await;
        assert!(history.is_some(), "Expected chat history to be present");
        let history = history.unwrap();
        assert_eq!(history.messages.len(), 1, "Expected one message in chat history");
        assert_eq!(history.messages[0].role, ChatRole::User, "Expected message role to be User");
        assert_eq!(history.messages[0].content, "Hello, Toodles!", "Expected message content to match");
    }
}