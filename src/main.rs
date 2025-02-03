use std::env;

use dotenv::dotenv;

use serenity::all::standard::CommandResult;
use serenity::all::validate_token;
use serenity::all::ChannelId;
use serenity::all::GuildRef;
use serenity::all::Message;
use serenity::all::Ready;
use serenity::all::User;
use serenity::model::voice;
use serenity::prelude::*;
use serenity::async_trait;
use songbird::SerenityInit;
use songbird::events::{Event, EventContext, EventHandler as VoiceEventHandler, TrackEvent};


fn get_user_voice_channel<'a>(i_user: &'a User, i_guild: &'a GuildRef) -> Option<&'a ChannelId> {
   i_guild
      .voice_states
      .get(&i_user.id)
      .and_then(|voice_state| voice_state.channel_id.as_ref())
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {

   async fn ready(&self, _: Context, ready: Ready) {
      println!("{} is connected!", ready.user.name);
   }

   async fn message(&self, ctx: Context, msg: Message) {
      if msg.content == "!ping" {
         if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
            println!("Error sending message: {why:?}");
         }
      }
      else if msg.content.contains("sweet caroline") {
         let guild = &msg.guild(&ctx.cache);
         match guild {
            None => (),
            Some(unwrapped_guild) => {
               let voice_channel = get_user_voice_channel(&msg.author, &unwrapped_guild);
               match voice_channel {
                  None => println!("User: {} is not in a voice channel!", &msg.author.name),
                  Some(channel) => println!("Voice channel ID: {}", channel)
               }
            }  
         }
      }
   }
}

struct TrackErrorNotifier;

#[async_trait]
impl VoiceEventHandler for TrackErrorNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(track_list) = ctx {
            for (state, handle) in *track_list {
                println!(
                    "Track {:?} encountered an error: {:?}",
                    handle.uuid(),
                    state.playing
                );
            }
        }

        None
    }
}

async fn join(ctx: &Context, msg: &Message) -> CommandResult {
   let guild = &msg.guild(&ctx.cache).unwrap();
   let connect_to = match get_user_voice_channel(&msg.author, guild) {
      Some(channel) => channel.to_owned(),
      None => return Ok(())
   };

   let manager = songbird::get(ctx)
      .await
      .expect("Songbird Voice client placed in at initialisation.")
      .clone();

    if let Ok(handler_lock) = manager.join(guild.id, connect_to).await {
      // Attach an event handler to see notifications of all track errors.
      let mut handler = handler_lock.lock().await;
      handler.add_global_event(TrackEvent::Error.into(), TrackErrorNotifier);
    }

    Ok(())
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
      Client::builder(&token, intents).event_handler(Handler).register_songbird().await.expect("Err creating client");
 
      // Start listening for events by starting a single shard
      if let Err(why) = client.start().await {
         println!("Client error: {why:?}");
      }
   }
   else {
      println!("Unable to validate the provided token! Failed to begin listening.");
   }
}
