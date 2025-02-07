use std::env;
use std::sync::Arc;

use dotenv::dotenv;

use poise::serenity_prelude as serenity;

use ::serenity::all::GatewayIntents;
use ::serenity::all::GuildRef;
use ::serenity::Client;
use serenity::all::validate_token;
use serenity::all::ChannelId;
use serenity::all::User;
use songbird::SerenityInit;

// Event related imports to detect track creation failures.
use songbird::events::{Event, EventContext, EventHandler as VoiceEventHandler, TrackEvent};

use tokio::time::Duration;

struct Data {} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

fn get_user_voice_channel<'a>(i_user: &'a User, i_guild: GuildRef<'a>) -> Option<ChannelId> {
    i_guild
        .voice_states
        .get(&i_user.id)
        .and_then(|voice_state| voice_state.channel_id)
}

async fn _join(ctx: &serenity::Context, guild_id: &serenity::GuildId, channel_id: &ChannelId) {
    let manager = songbird::get(&ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Ok(handler_lock) = manager.join(*guild_id, *channel_id).await {

        // Print message to log that the bot successfully joined the channel
        let _ = channel_id.say(&ctx.http, "Joined the voice channel!").await;

        // Attach an event handler to see notifications of all track errors.
        let mut handler = handler_lock.lock().await;
        handler.add_global_event(TrackEvent::Error.into(), TrackErrorNotifier);

        // After joining, leave after a timeout
        leave_after_timeout(ctx, guild_id, handler.clone()).await;
    }
}

async fn play(ctx: &serenity::Context, guild_id: &serenity::GuildId, channel_id: &ChannelId) -> Result<(), Error> {

    let manager = songbird::get(&ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Ok(handler_lock) = manager.join(*guild_id, *channel_id).await {

        // Print message to log that the bot successfully joined the channel
        channel_id.say(&ctx.http, "Joined the voice channel!").await.unwrap();

        // Attach an event handler to see notifications of all track errors.
        let mut handler = handler_lock.lock().await;
        handler.add_global_event(TrackEvent::Error.into(), TrackErrorNotifier);
        handler.add_global_event(TrackEvent::Play.into(), TrackStartNotifier);

        // Load an audio file (change the path to your audio file)
        let track = songbird::input::File::new("./caroline.mp3").into();

        // Play the audio
        handler.play(track);

        // Let the user know that the bot is playing the audio
        channel_id.say(&ctx.http, "Now playing your requested audio!").await.unwrap();

        // After joining, leave after a timeout
        leave_after_timeout(ctx, guild_id, handler.clone()).await;
    }
    
    Ok(())
}

async fn leave_after_timeout(ctx: &serenity::Context, guild_id: &serenity::GuildId, mut handler: songbird::Call) {
    // Define the timeout duration
    let timeout_duration = Duration::from_secs(3); // 10 seconds timeout
    tokio::time::sleep(timeout_duration).await;

    // Leave the voice channel after the timeout period
    if let Err(e) = handler.leave().await {
        println!("Failed to leave the voice channel: {}", e);
    } else {
        println!("Bot left the voice channel.");
    }
}

struct TrackErrorNotifier;

#[serenity::async_trait]
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

struct TrackStartNotifier;

#[serenity::async_trait]
impl VoiceEventHandler for TrackStartNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        println!("Track has started playing!");
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

async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Ready { data_about_bot, .. } => {
            println!("Logged in as {}", data_about_bot.user.name);
        }
        serenity::FullEvent::Message { new_message } => {
            if new_message.content.to_lowercase().contains("sweet caroline")
                && new_message.author.id != ctx.cache.current_user().id
            {
                println!("Message receive!");
                if let Some(channel_id) = get_user_voice_channel(&new_message.author, new_message.guild(&ctx.cache).unwrap()) {
                    play(ctx, &new_message.guild_id.unwrap(), &channel_id).await.unwrap();
                }
                else {
                    println!("User {} was not in a voice channel! Unable to join.", &new_message.author.name);
                }
            }
        }
        _ => {}
    }
    Ok(())
}

/// Displays your or another user's account creation date
#[poise::command(slash_command, prefix_command)]
async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    ctx.channel_id().say(&ctx.http(), "Pong!").await?;

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
        let intents = GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

        let framework = poise::Framework::builder()
            .options(poise::FrameworkOptions {
                commands: vec![ping()],
                event_handler: |ctx, event, framework, data| {
                    Box::pin(event_handler(ctx, event, framework, data))
                },
                ..Default::default()
            })
            .setup(|ctx, _ready, framework| {
                Box::pin(async move {
                    poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                    Ok(Data {})
                })
            })
            .build();

        // Create a new instance of the Client, logging in as a bot.
        let mut client = Client::builder(&token, intents)
            .framework(framework)
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
