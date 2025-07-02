
use std::error::Error;

use async_openai::{types::{ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage, ChatCompletionRequestSystemMessageContent, ChatCompletionRequestUserMessage, ChatCompletionRequestUserMessageContent, CreateChatCompletionRequestArgs}, Chat, Client};

use crate::models::ChatHistory;

static OPEN_AI_MODEL: &str = "gpt-3.5-turbo";

pub async fn ask_toodles(chat_history: ChatHistory) -> Result<String, Box<dyn Error>> {
    let client = Client::new();
    let request = CreateChatCompletionRequestArgs::default()
        .model(OPEN_AI_MODEL)
        .messages::<Vec<ChatCompletionRequestMessage>>(chat_history.into())
        .max_tokens(200u16)
        .build()?;

    let response = client.chat().create(request).await?;
    let reply = response.choices.get(0).and_then(|choice| choice.message.content.clone())
            .unwrap_or_else(|| "No response from Toodles".to_string());
    
    Ok(reply)
}


// Test
#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ChatHistory;
    #[tokio::test]
    async fn test_ask_toodles() {
        dotenv::dotenv().ok();
        let mut chat_history = ChatHistory::default();

        chat_history.add_system_message("You are Toodles the clown, a friendly and helpful AI assistant. Respond to user queries with humor and kindness.".to_string());
        chat_history.add_user_message("Hello, Toodles!".to_string());

        let response = ask_toodles(chat_history).await;
        assert!(response.is_ok(), "Expected a successful response, got an error: {:?}", response.err());

        let reply = response.unwrap();
        assert!(!reply.is_empty(), "Expected a non-empty response from Toodles");
        println!("Toodles replied: {}", reply);
    }
}