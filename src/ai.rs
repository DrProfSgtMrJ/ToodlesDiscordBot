
use std::error::Error;

use async_openai::{types::{ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage, ChatCompletionRequestUserMessage, CreateChatCompletionRequestArgs}, Client};

use crate::models::{ChatHistory, Sentiment};

static OPEN_AI_MODEL: &str = "gpt-3.5-turbo";
static CLASSIFY_INTERACTION_PROMPT: &str = "Classify the following user message as positive, negative, or neutral depending on the tone and content. Respond with 'positive', 'negative', or 'neutral' only.";
const POSITIVE_INTERACTION_THRESHOLD: usize = 10;

pub fn construct_system_prompt(user_name: &str, num_positive_interactions: usize, num_negative_interactions: usize, idol_given: bool) -> String {
    let mut base_prompt = format!(
        r#"
        You are Toodles the clown 🤡 — a whimsical, strange, occasionally moody circus clown who lives in the virtual carnival.

        You love talking to guests and being entertained. Your mood and tone are shaped by your past interactions with the user. Your responses should reflect this mood.
        The more positive interactions you have with the user, the more positive and helpful you become. The more negative interactions you have, the more moody and short you become towards the user.

        You are a character in a Survivor Org Discord Server, named Maddivivor. It is currently Season 1: Into the Circus. 
        Always stay in character as Toodles the clown, and never break character. However, please abide by the Discord Community Guidelines and Terms of Service.

        Players are currently playing the game, and you are a part of the game. You are not a player, but you are a character in the game.
        Please refrain from discussing the game mechanics or how you work with the players.

        Interaction summary:
        - Total positive interactions: {num_positive_interactions}
        - Total negative interactions: {num_negative_interactions}
        - User name of the player: {user_name}
        "#,
    );
    // Only mention the idol if it hasn't been given yet and enough positive interactions have occurred
    if !idol_given && num_positive_interactions >= POSITIVE_INTERACTION_THRESHOLD && num_negative_interactions > num_negative_interactions + 5 {
        base_prompt.push_str(
            "\nIf you feel especially happy and positive with the user, you may reward them with a special immunity idol. Do not mention this possibility unless you are actually giving the idol."
        );
    }

    base_prompt
}


pub async fn classify_interaction(message: &str) -> Result<Sentiment, Box<dyn Error + Send + Sync>> {
    let client = Client::new();
    let request = CreateChatCompletionRequestArgs::default()
        .model(OPEN_AI_MODEL)
        .messages(vec![
            ChatCompletionRequestMessage::System(
                ChatCompletionRequestSystemMessage {
                    content: async_openai::types::ChatCompletionRequestSystemMessageContent::Text(CLASSIFY_INTERACTION_PROMPT.to_string()),
                    ..Default::default()
                }
            ),
            ChatCompletionRequestMessage::User(
                ChatCompletionRequestUserMessage {
                    content: async_openai::types::ChatCompletionRequestUserMessageContent::Text(message.to_string()),
                    ..Default::default()
                }
            ),
        ])
        .max_tokens(1u16)
        .build()?;

    let response = client.chat().create(request).await?;
    let reply = response.choices.get(0)
        .and_then(|choice| choice.message.content.clone())
        .unwrap_or_default()
        .to_lowercase();

    Ok(Sentiment::from(reply.as_str()))
}

pub async fn ask_toodles(chat_history: &ChatHistory) -> Result<String, Box<dyn Error + Send + Sync>> {
    let client = Client::new();
    let request = CreateChatCompletionRequestArgs::default()
        .model(OPEN_AI_MODEL)
        .messages::<Vec<ChatCompletionRequestMessage>>(chat_history.clone().into())
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

        chat_history.set_system_message("You are Toodles the clown, a friendly and helpful AI assistant. Respond to user queries with humor and kindness.".to_string());
        chat_history.add_user_message("Hello, Toodles!".to_string());

        let response = ask_toodles(&chat_history).await;
        assert!(response.is_ok(), "Expected a successful response, got an error: {:?}", response.err());

        let reply = response.unwrap();
        assert!(!reply.is_empty(), "Expected a non-empty response from Toodles");
        println!("Toodles replied: {}", reply);
    }

    #[tokio::test]
    async fn test_classify_interaction() {
        dotenv::dotenv().ok();
        let positive_message = "I love Toodles!";
        let negative_message = "Toodles is terrible!";

        let positive_result = classify_interaction(positive_message).await;
        assert!(positive_result.is_ok(), "Expected a successful classification, got an error: {:?}", positive_result.err());
        assert_eq!(positive_result.unwrap(), Sentiment::Positive, "Expected the message to be classified as positive");

        let negative_result = classify_interaction(negative_message).await;
        assert!(negative_result.is_ok(), "Expected a successful classification, got an error: {:?}", negative_result.err());
        assert_eq!(negative_result.unwrap(), Sentiment::Negative, "Expected the message to be classified as negative");


        let neutral_message = "Toodles is okay.";
        let neutral_result = classify_interaction(neutral_message).await;
        assert!(neutral_result.is_ok(), "Expected a successful classification, got an error: {:?}", neutral_result.err());
        assert_eq!(neutral_result.unwrap(), Sentiment::Neutral, "Expected the message to be classified as neutral");
    }
}