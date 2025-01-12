use crate::utils::config::ENROLL_CHANNEL;
use crate::{Context, Error};

// makes sure the command was sent in the enroll channel
pub async fn enroll_channel(ctx: Context<'_>) -> Result<bool, Error> {
    let guild_channel = match ctx.guild_channel().await {
        Some(channel) => channel,
        None => {
            println!("Unable to get the guild_channel");
            return Ok(false);
        }
    };

    let channel_id = guild_channel.id.get();

    let enroll_channel_id = match ctx.data().config_data.channels.get(ENROLL_CHANNEL) {
        Some(id) => id,
        None => {
            println!("Unable to get the enroll channel ID");
            return Ok(false);
        }
    };

    if channel_id == *enroll_channel_id {
        Ok(true)
    } else {
        Ok(false)
    }
}
