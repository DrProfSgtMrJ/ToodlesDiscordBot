
use std::error::Error;

use async_openai::{types::{ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage, ChatCompletionRequestSystemMessageContent, ChatCompletionRequestUserMessage, ChatCompletionRequestUserMessageContent, CreateChatCompletionRequestArgs}, Chat, Client};

static OPEN_AI_MODEL: &str = "gpt-3.5-turbo";

pub async fn ask_toodles(user_message: &str, system_prompt: &str) -> Result<String, Box<dyn Error>> {
    let client = Client::new();
    let request = CreateChatCompletionRequestArgs::default()
        .model(OPEN_AI_MODEL)
        .messages(vec![
            ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
                content: ChatCompletionRequestSystemMessageContent::Text(system_prompt.to_string()),
                name: None,
            }),
            ChatCompletionRequestMessage::User(
                ChatCompletionRequestUserMessage {
                    content: ChatCompletionRequestUserMessageContent::Text(user_message.to_string()),
                    name: None,
                }
            )
        ])
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

    #[tokio::test]
    async fn test_ask_toodles() {
        dotenv::dotenv().ok();
        let user_message = "Hello, Toodles!";
        let system_prompt = "You are Toodles the clown, a friendly and helpful AI assistant. Respond to user queries with humor and kindness.";

        let response = ask_toodles(user_message, system_prompt).await;
        assert!(response.is_ok(), "Expected a successful response, got an error: {:?}", response.err());

        let reply = response.unwrap();
        assert!(!reply.is_empty(), "Expected a non-empty response from Toodles");
        println!("Toodles replied: {}", reply);
    }
}