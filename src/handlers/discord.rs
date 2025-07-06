use std::sync::Arc;

use serenity::all::Member;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;


use crate::handlers::handle_message::handle_message;
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

        if msg.content.starts_with(&self.prefix) {
            handle_message(&self.prefix, ctx, msg, self.chat_history_store.clone(), self.user_interaction_store.clone()).await.expect("Failed to handle message");
        }
    }

    async fn guild_member_addition(&self, ctx: Context, new_member: Member) {
        // This function can be used to handle new member additions
        println!("New member added: {}", new_member.user.name);
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("Bot is ready!");
    }
}

impl DiscordHandler {

    pub fn new(prefix: String, chat_history_store: Arc<dyn ChatHistoryStore + Send + Sync>, user_interaction_store: Arc<dyn UserInteractionStore + Send + Sync>) -> Self {
        DiscordHandler { prefix, chat_history_store, user_interaction_store }
    }
}