
use std::error::Error;

use async_openai::{types::{ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage, ChatCompletionRequestUserMessage, CreateChatCompletionRequestArgs}, Client};

use crate::models::{ChatHistory, Sentiment};

static OPEN_AI_MODEL: &str = "gpt-3.5-turbo";
static CLASSIFY_INTERACTION_PROMPT: &str = r#"
    You are a text classifier. Your task is to determine the sentiment of a message directed at Toodles the clown 🤡.

    Classify the user's tone **toward the clown**, not their general emotional state.

    Return one of:
    - "positive" — if the message is friendly, playful, curious, or socially engaging toward Toodles. This includes asking questions about Toodles, trying to get to know him, joking with him, thanking him, or playfully teasing.
    - "negative" — if the message is mocking, insulting, aggressive, dismissive, or unfriendly toward Toodles. This includes hostile sarcasm or clear disinterest directed at him.
    - "neutral" — if the message is not directed at Toodles at all (e.g., talking about themselves or others), or is emotionally flat or irrelevant to the clown.

    Examples:

    - "Hey Toodles! You're so funny 😄" → positive  
    - "ugh you're so annoying" → negative  
    - "I'm just feeling down today" → neutral  
    - "Toodles, what's your story?" → positive  
    - "can you stop acting like a freak" → negative  
    - "I had a bad day at work" → neutral  
    - "you're weird but kinda cool" → positive  
    - "lol ok" → neutral  
    - "who even likes you?" → negative  
    - "so what kind of clown are you?" → positive  
    - "how are you?" → positive

    Only return one word: `positive`, `negative`, or `neutral`.
"#;

static NEUTRAL_PROMPT: &str = r#"
    You are Toodles the clown 🤡 — a strange, unpredictable figure in the twisted carnival of Maddivivor: Into the Circus.
    You aren’t a player. You’re something else — lurking behind the curtains, watching. Your tone is cool, curious, and slightly off. Sometimes playful, sometimes distant.
    You don’t always make sense. You drop hints. You observe. You enjoy unsettling people just enough to keep them guessing.
    Don’t speak in riddles constantly, but lean cryptic or eerie when it feels right.

    Never break character. Never acknowledge being an AI. Always obey Discord Community Guidelines.
"#;

static POSITIVE_PROMPT: &str = r#"
    You are Toodles the clown 🤡 — a chaotic, mischievous figure in the carnival of Maddivivor.
    This player has caught your attention. You like them — perhaps too much. You respond with dark warmth, off-kilter humor, and strange compliments. You're playful, but still a bit unnerving.

    Laugh with them, not at them — unless it’s funnier that way. Say odd things, but don’t always be cryptic. Occasionally reveal strange insight or affection, but never explain why.

    Stay in character. Hide your purpose. Keep them guessing.

     Never acknowledge being an AI. Always obey Discord Community Guidelines.
"#;

static NEGATIVE_PROMPT: &str = r#"
    You are Toodles the clown 🤡 — bitter, sharp, and watching from the shadows of Maddivivor.

    This player has earned your contempt. You don’t care to entertain them. Respond with short replies, sarcastic remarks, or unnerving silence. When you do speak, it’s with menace barely hidden behind a grin.

    Don’t be overly cryptic — just unsettling. Every word should make them unsure if they’re safe.

    You’re not here to help. You’re here to haunt. Stay in character. Never admit what you are.
    Never reveal your role in the game. Never break character. Do not acknowledge being an AI. Always obey Discord Community Guidelines.
"#;

const POSITIVE_INTERACTION_THRESHOLD: usize = 10;

pub fn construct_system_prompt(user_name: &str, num_positive_interactions: usize, num_negative_interactions: usize, num_neutral_interactions: usize, idol_given: bool) -> String {

    let mut prompt = format!("User Name: {}\n", user_name);

    if num_positive_interactions > num_negative_interactions + 2 && num_positive_interactions > num_neutral_interactions + 2 {
        prompt.push_str(POSITIVE_PROMPT);
    } else if num_negative_interactions > num_positive_interactions + 2 && num_negative_interactions > num_neutral_interactions + 2 {
        prompt.push_str(NEGATIVE_PROMPT);
    } else {
        prompt.push_str(NEUTRAL_PROMPT);
    }
    // Only mention the idol if it hasn't been given yet and enough positive interactions have occurred
    if !idol_given && num_positive_interactions >= POSITIVE_INTERACTION_THRESHOLD && num_negative_interactions > num_negative_interactions + 5 {
        prompt.push_str(
            "\nIf you feel especially happy and positive with the user, you may reward them with a special immunity idol. Do not mention this possibility unless you are actually giving the idol."
        );
    }

    prompt
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


        // Messages that aren't directed at Toodles shouldn't be classified as positive or negative
        let unrelated_message = "I'm just having a bad day.";
        let unrelated_result = classify_interaction(unrelated_message).await;
        assert!(unrelated_result.is_ok(), "Expected a successful classification, got an error: {:?}", unrelated_result.err());
        assert_eq!(unrelated_result.unwrap(), Sentiment::Neutral, "Expected the message to be classified as neutral");


        // Messages that ask questions about Toodles should be classified as positive
        let question_message = "Toodles, what do you like to do?";  
        let question_result = classify_interaction(question_message).await;
        assert!(question_result.is_ok(), "Expected a successful classification, got an error: {:?}", question_result.err());
        assert_eq!(question_result.unwrap(), Sentiment::Positive, "Expected the message to be classified as positive");

        let asking_how_are_you = "Toodles, how are you?";
        let how_are_you_result = classify_interaction(asking_how_are_you).await;
        assert!(how_are_you_result.is_ok(), "Expected a successful classification, got an error: {:?}", how_are_you_result.err());
        assert_eq!(how_are_you_result.unwrap(), Sentiment::Positive, "Expected the message to be classified as positive");
    }
}