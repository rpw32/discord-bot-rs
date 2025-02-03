use std::env;

use dotenv::dotenv;

use serenity::all::validate_token;
use serenity::all::ChannelId;
use serenity::all::GuildRef;
use serenity::all::Message;
use serenity::all::User;
use serenity::prelude::*;
use serenity::async_trait;

fn get_user_voice_channel<'a>(i_user: &'a User, i_guild: &'a GuildRef) -> Option<&'a ChannelId> {
   let iterator = i_guild.voice_states.iter();
   for state in iterator {
      if state.0.eq(&i_user.id) {
          return state.1.channel_id.as_ref();
      }
   }
   None
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("Error sending message: {why:?}");
            }
        }
        else if msg.content.contains("sweet caroline") {
            let guild = &msg.guild(&ctx.cache).unwrap();
            let voice_channel = get_user_voice_channel(&msg.author, &guild);
            match voice_channel {
               None => println!("User is not in a voice channel!"),
               Some(channel) => println!("Voice channel ID: {}", channel)
            }
        }
    }
}

#[tokio::main]
async fn main() {

   // Load the environment variables from the dotenv file
   dotenv().ok();

   // Login with a bot token from the environment
   let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

   let validation_result = validate_token(&token);
   if validation_result.is_ok() {
      // Set gateway intents, which decides what events the bot will be notified about
      let intents = GatewayIntents::GUILD_MESSAGES 
         | GatewayIntents::GUILDS
         | GatewayIntents::DIRECT_MESSAGES
         | GatewayIntents::MESSAGE_CONTENT
         | GatewayIntents::GUILD_VOICE_STATES;
 
      // Create a new instance of the Client, logging in as a bot.
      let mut client =
      Client::builder(&token, intents).event_handler(Handler).await.expect("Err creating client");
 
      // Start listening for events by starting a single shard
      if let Err(why) = client.start().await {
         println!("Client error: {why:?}");
      }
   }
   else {
      println!("Unable to validate the provided token! Failed to begin listening.");
   }



}
