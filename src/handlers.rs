use serenity::all::EditMessage;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;




#[derive(Debug)]
pub struct DiscordHandler {
    pub prefix: String,
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

    pub fn new(prefix: String) -> Self {
        DiscordHandler { prefix }
    }

    async fn handle_command(&self, ctx: Context, msg: Message) {
        // This function can be used to handle specific commands
        if msg.content.starts_with(&self.prefix) {
            println!("Handling command: {}", msg.content);

            // Send temporary "thinking" message
            let mut thinking_msg = match msg.reply(&ctx.http, "🤡 Toodles is thinking...").await {
                Ok(m) => m,
                Err(why) => {
                    println!("Error sending thinking message: {:?}", why);
                    return;
                }
            };

            // Add a delay to simulate processing time
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;

            // Edit the thinking message to show the result
            if let Err(why) = thinking_msg.edit(&ctx.http, EditMessage::new().content("🤡 Toodles has thought about it!")).await {
                println!("Error editing thinking message: {:?}", why);
                return;
            }
            // Add command handling logic here

            let prompt = msg.content.trim_start_matches(&self.prefix);
            println!("Command received: {}", prompt);

            if let Some(guild_id) = msg.guild_id {
                if let Ok(member) = guild_id.member(&ctx.http, msg.author.id).await {
                    println!("Member info: {:?}", member);
                    println!("Member roles: {:?}", member.roles);
                }
            }
        } else {
            println!("Received message without command prefix: {}", msg.content);
        }
    }
}