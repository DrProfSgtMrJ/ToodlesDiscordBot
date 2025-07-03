use std::sync::Arc;

use serenity::all::EditMessage;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;


use crate::ai::ask_toodles;
use crate::store::ChatHistoryStore;


pub struct DiscordHandler {
    pub prefix: String,
    pub chat_history_store: Arc<dyn ChatHistoryStore + Send + Sync>
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

    pub fn new(prefix: String, chat_history_store: Arc<dyn ChatHistoryStore + Send + Sync>) -> Self {
        DiscordHandler { prefix, chat_history_store }
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
            let user_message = msg.content.clone();
            let username = &msg.author.name;

            let mut chat_history = self.chat_history_store.get_chat_history(&user_id).await;

            if chat_history.messages.is_empty() {
                chat_history.add_system_message("You are Toodles the clown, a friendly and helpful AI assistant. Respond to user queries with humor and kindness.".to_string());
            }
            chat_history.add_user_message(user_message.clone());
            match ask_toodles(&chat_history).await {
                Ok(reply) => {
                    if let Err(why) = thinking_msg.edit(&ctx.http, EditMessage::new().content(&reply)).await {
                        println!("Error sending response message: {:?}", why);
                    }
                    // Add to the chat history store
                    //self.chat_history_store.add_chat_message(user_id, );
                    //self.chat_history_store.add_chat_message(user_id, message)
                    // Add the assistant's reply to the chat history
                    chat_history.clone().add_assistant_message(reply.clone());
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