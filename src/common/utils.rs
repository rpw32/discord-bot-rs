use poise::serenity_prelude as serenity;

pub fn get_user_voice_channel<'a>(
    i_user: &'a serenity::User,
    i_guild: serenity::GuildRef<'a>,
) -> Option<serenity::ChannelId> {
    i_guild
        .voice_states
        .get(&i_user.id)
        .and_then(|voice_state| voice_state.channel_id)
}
