use std::sync::Arc;

use serenity::all::{Context, EditMessage, Message};

use crate::{ai::{ask_toodles, classify_interaction, construct_system_prompt}, models::Sentiment, store::{ChatHistoryStore, UserInteractionStore}};


pub async fn handle_message(
    prefix: &str,
    ctx: Context,
    msg: Message,
    chat_history_store: Arc<dyn ChatHistoryStore + Send + Sync>,
    user_interaction_store: Arc<dyn UserInteractionStore + Send + Sync>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let user_id = msg.author.id.to_string();
    let username = &msg.author.name;
    let user_message = msg.content.strip_prefix(prefix).unwrap_or(&msg.content).to_string();


    let mut thinking_msg = match msg.reply(&ctx.http, "🤡 Toodles is thinking...").await {
        Ok(m) => m,
        Err(why) => {
            return Err(Box::new(why));
        }
    };

    let sentiment = classify_interaction(&user_message).await?;
    let mut chat_history = chat_history_store.get_chat_history(&user_id).await;
    let mut user_interaction = user_interaction_store.get_user_interaction(&user_id).await;

    match sentiment {
        Sentiment::Positive => {
            user_interaction.increment_positive();
            user_interaction_store.increment_positive_interaction(&user_id).await;
            println!("User {} sent a positive message: {}", username, user_message);
        },
        Sentiment::Negative => {
            user_interaction.increment_negative();
            user_interaction_store.increment_negative_interaction(&user_id).await;
            println!("User {} sent a negative message: {}", username, user_message);
        },
        Sentiment::Neutral => {
            user_interaction.increment_neutral();
            user_interaction_store.increment_neutral_interaction(&user_id).await;
            println!("User {} sent a neutral message: {}", username, user_message);
        }
    }

    let system_message = construct_system_prompt(&username, user_interaction.num_positive, user_interaction.num_negative, user_interaction.num_neutral, false);
    println!("Constructed system message: {}", system_message);
    chat_history.set_system_message(system_message);
    chat_history.add_user_message(user_message.clone());

    match ask_toodles(&chat_history).await {
        Ok(reply) => {
            if let Err(why) = thinking_msg.edit(&ctx.http, EditMessage::new().content(&reply)).await {
                println!("Error sending response message: {:?}", why);
            }

            // Add to the chat history store
            chat_history_store.add_user_message(&user_id, user_message.clone()).await;
            chat_history_store.add_assistant_message(&user_id, reply.clone()).await;
        },
        Err(e) => {
            if let Err(why) = thinking_msg.edit(&ctx.http, EditMessage::new().content("🤡 Toodles encountered an error while thinking!")).await {
                println!("Error sending error message: {:?}", why);
                return Err(Box::new(why));
            }
            return Err(e);
        }
    }

    Ok(())
}