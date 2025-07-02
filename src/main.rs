mod handlers;
mod ai;
mod models;
mod chat_hisotry_store;

use dotenv::dotenv;

use handlers::DiscordHandler;
use serenity::{all::GatewayIntents, Client};




#[tokio::main]
async fn main() {
    // Load environment variables from .env file
    dotenv().ok();

    let token = std::env::var("DISCORD_TOODLE_BOT_TOKEN")
        .expect("Expected DISCORD_TOODLE_BOT_TOKEN in .env file");

    let handler = DiscordHandler::new("!toodles".to_string());

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(&token, intents).event_handler(handler).await.expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }

    println!("Bot is running!");
}
