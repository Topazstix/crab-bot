use poise::serenity_prelude as serenity;
use poise::serenity_prelude::FullEvent::{GuildMemberAddition, Message, Ready, VoiceStateUpdate};
use poise::serenity_prelude::{CacheHttp, CreateMessage, Mentionable};

use crate::{Data, Error};

pub async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        // Bot successfully logged in
        Ready { data_about_bot, .. } => {
            println!("Logged in as {}", data_about_bot.user.name);
        }

        // New member joined the server
        GuildMemberAddition { new_member, .. } => {
            let member_guild = new_member.guild_id.get();
            if data.config_data.guild.main.guild_id == member_guild {
                let join_message = CreateMessage::new().content(format!(
                    "Welcome to {}, {}! Thanks for joining us! 🎉",
                    ctx.http().get_guild(new_member.guild_id).await?.name,
                    new_member.user.mention(),
                ));
                new_member
                    .user
                    .create_dm_channel(ctx.http.http())
                    .await?
                    .send_message(ctx.http.http(), join_message)
                    .await?;
            }
            println!(
                "{} Joined Server {} ",
                new_member.user.name,
                ctx.http().get_guild(new_member.guild_id).await?.name
            );
        }
        _ => {}
    }
    Ok(())
}
