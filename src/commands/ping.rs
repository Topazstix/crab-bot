use crate::{Context, Error};

// quick simple way to confirm it's working
#[poise::command(slash_command, ephemeral, hide_in_help)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    ctx.reply("Pong!").await?;
    Ok(())
}
