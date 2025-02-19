mod commands;
mod common;

use common::{Context, Data, Error};

use std::env;

use dotenv::dotenv;

use poise::serenity_prelude as serenity;

use songbird::SerenityInit;

async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    _data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Ready { data_about_bot, .. } => {
            println!("Logged in as {}", data_about_bot.user.name);
        }
        serenity::FullEvent::Message { new_message } => {
            if new_message
                .content
                .to_lowercase()
                .contains("sweet caroline")
                && new_message.author.id != ctx.cache.current_user().id
            {
                println!("Message receive!");
                if let Some(channel_id) = common::utils::get_user_voice_channel(
                    &new_message.author,
                    new_message.guild(&ctx.cache).unwrap(),
                ) {
                    commands::voice::play(
                        ctx,
                        &new_message.guild_id.unwrap(),
                        &new_message.channel_id,
                        &channel_id,
                    )
                    .await
                    .unwrap();
                } else {
                    println!(
                        "User {} was not in a voice channel! Unable to join.",
                        &new_message.author.name
                    );
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
    ctx.say("Pong!").await?;

    Ok(())
}

#[poise::command(slash_command, prefix_command)]
async fn timeout(ctx: Context<'_>) -> Result<(), Error> {
    // Create the modal with components (e.g., text input)
    let modal = serenity::CreateModal::new("my_modal", "Please enter some text:").components(vec![
        serenity::CreateActionRow::InputText(serenity::CreateInputText::new(
            serenity::InputTextStyle::Short,
            "label",
            "custom_id",
        )),
    ]);

    // Send the modal using Serenity API
    ctx.http()
        .create_interaction_response(&ctx, |r| {
            r.kind(InteractionResponseType::Modal).modal(modal)
        })
        .await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    // Load the environment variables from the dotenv file
    dotenv().ok();

    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let validation_result = serenity::validate_token(&token);
    if validation_result.is_ok() {
        // Set gateway intents, which decides what events the bot will be notified about
        let intents =
            serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

        let framework = poise::Framework::builder()
            .options(poise::FrameworkOptions {
                commands: vec![ping()],
                event_handler: |ctx: &::serenity::prelude::Context, event, framework, data| {
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
        let mut client = serenity::Client::builder(&token, intents)
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
    } else {
        println!("Unable to validate the provided token! Failed to begin listening.");
    }
}
