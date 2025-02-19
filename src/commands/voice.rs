use poise::serenity_prelude as serenity;

// Event related imports to detect track creation failures.
use songbird::events::{Event, EventContext, EventHandler as VoiceEventHandler, TrackEvent};

use tokio::time::Duration;

pub async fn play(
    ctx: &serenity::Context,
    guild_id: &serenity::GuildId,
    msg_channel_id: &serenity::ChannelId,
    voice_channel_id: &serenity::ChannelId,
) -> Result<(), serenity::Error> {
    let manager = songbird::get(&ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Ok(handler_lock) = manager.join(*guild_id, *voice_channel_id).await {
        // Print message to log that the bot successfully joined the channel
        msg_channel_id
            .say(&ctx.http, "Joined the voice channel!")
            .await
            .unwrap();

        // Attach an event handler to see notifications of all track errors.
        let mut handler = handler_lock.lock().await;
        handler.add_global_event(TrackEvent::End.into(), TrackEndNotifier);

        // Load an audio file (change the path to your audio file)
        let track = songbird::input::File::new("./caroline.mp3").into();

        // Play the audio
        handler.play(track);

        // Let the user know that the bot is playing the audio
        msg_channel_id
            .say(&ctx.http, "Now playing your requested audio!")
            .await
            .unwrap();

        // After joining, leave after a timeout
        leave_after_timeout(ctx, guild_id, handler.clone()).await;
    }

    Ok(())
}

async fn leave_after_timeout(
    _ctx: &serenity::Context,
    _guild_id: &serenity::GuildId,
    mut handler: songbird::Call,
) {
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

struct TrackEndNotifier;

#[serenity::async_trait]
impl VoiceEventHandler for TrackEndNotifier {
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<Event> {
        println!("Track has stopped playing!");
        None
    }
}
