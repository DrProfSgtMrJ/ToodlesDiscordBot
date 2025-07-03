use async_trait::async_trait;
use std::collections::HashMap;
use tokio::sync::RwLock;
use sqlx::{PgPool, Row};
use std::sync::Arc;


use crate::models::{ChatHistory, ChatMessage, ChatRole};

#[async_trait]
pub trait ChatHistoryStore {
    async fn add_chat_message(&self, user_id: &str, message: ChatMessage);
    async fn get_chat_history(&self, user_id: &str) -> ChatHistory;
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

    async fn get_chat_history(&self, user_id: &str) -> ChatHistory {
        let store = self.store.read().await;
        store.get(user_id).cloned().unwrap_or_default()
    }
}

pub struct PostgresChatHistoryStore {
    pool: PgPool,
}

impl PostgresChatHistoryStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ChatHistoryStore for PostgresChatHistoryStore {
    async fn add_chat_message(&self, user_id: &str, message: ChatMessage) {
        let query = "INSERT INTO chat_messages (user_id, role, content) VALUES ($1, $2, $3)";
        let role = match message.role {
            ChatRole::User => "user",
            ChatRole::Assistant => "assistant",
            ChatRole::System => "system",
        };

        sqlx::query(query)
            .bind(user_id)
            .bind(role)
            .bind(message.content)
            .execute(&self.pool)
            .await
            .unwrap();
    }

    async fn get_chat_history(&self, user_id: &str) -> ChatHistory {
        let query = "SELECT role, content FROM chat_messages WHERE user_id = $1 ORDER BY timestamp ASC";
        let rows = sqlx::query(query)
            .bind(user_id)
            .fetch_all(&self.pool)
            .await
            .ok();

        let messages = rows
            .unwrap_or_default()
            .into_iter()
            .map(|row| {
                let role_str: String = row.get("role");
                let role = match role_str.as_str() {
                    "user" => ChatRole::User,
                    "assistant" => ChatRole::Assistant,
                    "system" => ChatRole::System,
                    _ => ChatRole::System, // Default to System if role is unknown
                };

                let content: String = row.get("content");
                ChatMessage {
                    role,
                    content,
                }
            })
            .collect();

        ChatHistory { messages }
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
        assert!(store.get_chat_history(user_id).await.messages.is_empty());

        // Create a message and add it to the store
        let message = ChatMessage {
            role: ChatRole::User,
            content: "Hello, Toodles!".to_string(),
        };
        store.add_chat_message(user_id, message).await;

        // Retrieve the chat history for the user
        let history = store.get_chat_history(user_id).await;
        assert_eq!(history.messages.len(), 1, "Expected one message in chat history");
        assert_eq!(history.messages[0].role, ChatRole::User, "Expected message role to be User");
        assert_eq!(history.messages[0].content, "Hello, Toodles!", "Expected message content to match");
    }

    #[tokio::test]
    async fn test_postgres_chat_history_store() {
        // Don't have password and username here
        let pool = PgPool::connect("postgres://user:password!@localhost/toodles").await.unwrap();
        let store = PostgresChatHistoryStore::new(pool);
        let user_id = "test_user";

        let chat_history = store.get_chat_history(user_id).await;
        assert!(chat_history.messages.is_empty(), "Expected empty chat history for new user");

        // Create a message and add it to the store
        let message = ChatMessage {
            role: ChatRole::User,
            content: "Hello, Toodles!".to_string(),
        };
        store.add_chat_message(user_id, message).await;

        // Retrieve the chat history for the user
        let history = store.get_chat_history(user_id).await;
        assert_eq!(history.messages.len(), 1, "Expected one message in chat history");
        assert_eq!(history.messages[0].role, ChatRole::User, "Expected message role to be User");
        assert_eq!(history.messages[0].content, "Hello, Toodles!", "Expected message content to match");
    }
}