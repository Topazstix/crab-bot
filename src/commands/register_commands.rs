use crate::{Context, Error};

// admin command to register commands typically for debug use
#[poise::command(hide_in_help, guild_only, ephemeral, slash_command, owners_only)]
pub async fn register_commands(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}
