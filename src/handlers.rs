use std::sync::Arc;

use serenity::all::EditMessage;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;


use crate::ai::{ask_toodles, classify_interaction, construct_system_prompt};
use crate::models::Sentiment;
use crate::store::{ChatHistoryStore, UserInteractionStore};


pub struct DiscordHandler {
    pub prefix: String,
    pub chat_history_store: Arc<dyn ChatHistoryStore + Send + Sync>,
    pub user_interaction_store: Arc<dyn UserInteractionStore + Send + Sync>,
}

#[async_trait]
impl EventHandler for DiscordHandler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            // Ignore messages from bots
            return;
        }

        self.handle_command(ctx, msg).await;
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("Bot is ready!");
    }
}

impl DiscordHandler {

    pub fn new(prefix: String, chat_history_store: Arc<dyn ChatHistoryStore + Send + Sync>, user_interaction_store: Arc<dyn UserInteractionStore + Send + Sync>) -> Self {
        DiscordHandler { prefix, chat_history_store, user_interaction_store }
    }

    async fn handle_command(&self, ctx: Context, msg: Message) {
        // This function can be used to handle specific commands
        if msg.content.starts_with(&self.prefix) {
            // Send temporary "thinking" message
            let mut thinking_msg = match msg.reply(&ctx.http, "🤡 Toodles is thinking...").await {
                Ok(m) => m,
                Err(why) => {
                    println!("Error sending thinking message: {:?}", why);
                    return;
                }
            };

            let user_id = msg.author.id.to_string();
            let user_message = msg.content.strip_prefix(&self.prefix).unwrap_or(&msg.content).to_string();
            // Classify the interaction
            let sentiment = classify_interaction(&user_message).await.expect("Failed to classify interaction");

            // Get previous interaction count
            let username = &msg.author.name;

            let mut chat_history = self.chat_history_store.get_chat_history(&user_id).await;
            let mut user_interaction = self.user_interaction_store.get_user_interaction(&user_id).await;

            // Increment interaction counts based on classification
            // Also store the updated number
            match sentiment {
                Sentiment::Positive => {
                    user_interaction.increment_positive();
                    self.user_interaction_store.increment_positive_interaction(&user_id).await;
                },
                Sentiment::Negative => {
                    user_interaction.increment_negative();
                    self.user_interaction_store.increment_negative_interaction(&user_id).await;
                },
                Sentiment::Neutral => {
                    user_interaction.increment_neutral();
                    self.user_interaction_store.increment_neutral_interaction(&user_id).await;
                },
            }

            let system_message = construct_system_prompt(&username, user_interaction.num_positive, user_interaction.num_negative, false);
            chat_history.set_system_message(system_message);
            chat_history.add_user_message(user_message.clone());

            println!("Chat history for user {}: {:?}", user_id, chat_history);

            match ask_toodles(&chat_history).await {
                Ok(reply) => {
                    if let Err(why) = thinking_msg.edit(&ctx.http, EditMessage::new().content(&reply)).await {
                        println!("Error sending response message: {:?}", why);
                    }

                    // Add to the chat history store
                    self.chat_history_store.add_user_message(&user_id, user_message.clone()).await;
                    self.chat_history_store.add_assistant_message(&user_id, reply.clone()).await;
                },
                Err(e) => {
                    println!("Error asking Toodles: {:?}", e);
                    if let Err(why) = thinking_msg.edit(&ctx.http, EditMessage::new().content("🤡 Toodles encountered an error while thinking!")).await {
                        println!("Error sending error message: {:?}", why);
                    }
                    return;
                }
            }
        } else {
            println!("Received message without command prefix: {}", msg.content);
        }
    }
}