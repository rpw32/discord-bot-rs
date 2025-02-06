use std::env;

use dotenv::dotenv;

use poise::serenity_prelude as serenity;

use ::serenity::all::GatewayIntents;
use ::serenity::all::GuildRef;
use ::serenity::Client;
use serenity::all::validate_token;
use serenity::all::ChannelId;
use serenity::all::User;
use songbird::SerenityInit;

struct Data {} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

fn _get_user_voice_channel<'a>(i_user: &'a User, i_guild: &'a GuildRef) -> Option<&'a ChannelId> {
    i_guild
        .voice_states
        .get(&i_user.id)
        .and_then(|voice_state| voice_state.channel_id.as_ref())
}

// async fn join(songbird_manager: Option<Arc<Songbird>>, guild_id: GuildId, channel_id: ChannelId) {
//     let manager = songbird_manager
//         .expect("Songbird Voice client placed in at initialisation.")
//         .clone();

//     if let Ok(handler_lock) = manager.join(guild_id, channel_id).await {
//         // Attach an event handler to see notifications of all track errors.
//         let mut handler = handler_lock.lock().await;
//         handler.add_global_event(TrackEvent::Error.into(), TrackErrorNotifier);
//     }
// }

/// Displays your or another user's account creation date
#[poise::command(prefix_command)]
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
        let intents = GatewayIntents::non_privileged();

        let framework = poise::Framework::builder()
            .options(poise::FrameworkOptions {
                commands: vec![ping()],
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
