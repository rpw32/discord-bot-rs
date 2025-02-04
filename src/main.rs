use std::env;
use std::sync::Arc;

use dotenv::dotenv;

use serenity::all::validate_token;
use serenity::all::ChannelId;
use serenity::all::GuildId;
use serenity::all::GuildRef;
use serenity::all::Message;
use serenity::all::Ready;
use serenity::all::User;
use serenity::async_trait;
use serenity::prelude::*;
use songbird::events::{Event, EventContext, EventHandler as VoiceEventHandler, TrackEvent};
use songbird::SerenityInit;
use songbird::Songbird;
use tokio::task::LocalSet;

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
        } else if msg.content.contains("sweet caroline") {
            let local = LocalSet::new();
            //let context = ctx.clone();
            local.spawn_local(async move {
                let guild = &msg.guild(&ctx.cache).unwrap();
                let connect_to = match get_user_voice_channel(&msg.author, guild) {
                    Some(channel) => channel.to_owned(),
                    None => return (),
                };

                let guild_id = guild.id;
                let manager = songbird::get(&ctx).await;
                join(manager, guild_id, connect_to).await;
            });
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

async fn join(songbird_manager: Option<Arc<Songbird>>, guild_id: GuildId, channel_id: ChannelId) {
    let manager = songbird_manager
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Ok(handler_lock) = manager.join(guild_id, channel_id).await {
        // Attach an event handler to see notifications of all track errors.
        let mut handler = handler_lock.lock().await;
        handler.add_global_event(TrackEvent::Error.into(), TrackErrorNotifier);
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
        let mut client = Client::builder(&token, intents)
            .event_handler(Handler)
            .register_songbird()
            .await
            .expect("Err creating client");

        tokio::spawn(async move {
            let _ = client
                .start()
                .await
                .map_err(|why| println!("Client ended: {:?}", why));
        });

        let _signal_err = tokio::signal::ctrl_c().await;
        println!("Received Ctrl-C, shutting down.");
        // // Start listening for events by starting a single shard
        // if let Err(why) = client.start().await {
        //    println!("Client error: {why:?}");
        // }
    } else {
        println!("Unable to validate the provided token! Failed to begin listening.");
    }
}
