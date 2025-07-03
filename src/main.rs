mod handlers;
mod ai;
mod models;
mod store;

use std::sync::Arc;

use dotenv::dotenv;

use handlers::DiscordHandler;
use serenity::{all::GatewayIntents, Client};
use sqlx::PgPool;




#[tokio::main]
async fn main() {
    // Load environment variables from .env file
    dotenv().ok();

    let token = std::env::var("DISCORD_TOODLE_BOT_TOKEN")
        .expect("Expected DISCORD_TOODLE_BOT_TOKEN in .env file");
    let app_env = std::env::var("APP_ENV")
        .unwrap_or_else(|_| "development".to_string());

    let chat_history_store: Arc<dyn store::ChatHistoryStore + Send + Sync> = match app_env.as_str() {
        "development" => Arc::new(store::InMemoryChatHistoryStore::new()),
        "production" => {
            let db_url = std::env::var("DATABASE_URL")
                .expect("Expected DATABASE_URL in .env file for production");
            let pool = PgPool::connect(&db_url).await.expect("Failed to connect to database");
            Arc::new(store::PostgresChatHistoryStore::new(pool)) 
        },
        _ => panic!("Unknown APP_ENV: {}", app_env),
    };

    let user_interaction_store: Arc<dyn store::UserInteractionStore + Send + Sync> = match app_env.as_str() {
        "development" => Arc::new(store::InMemoryUserInteractionStore::new()),
        "production" => {
            let db_url = std::env::var("DATABASE_URL")
                .expect("Expected DATABASE_URL in .env file for production");
            let pool = PgPool::connect(&db_url).await.expect("Failed to connect to database");
            Arc::new(store::PostgresUserInteractionStore::new(pool))
        },
        _ => panic!("Unknown APP_ENV: {}", app_env),
    };
    let handler = DiscordHandler::new("!toodles".to_string(), chat_history_store, user_interaction_store);

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(&token, intents).event_handler(handler).await.expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }

    println!("Bot is running!");
}
