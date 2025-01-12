use crate::{Context, Error, REMOVE_ROLE_ID};

// makes sure the user has the remove role
pub async fn remove_role(ctx: Context<'_>) -> Result<bool, Error> {
    if let Some(role_id) = ctx.data().config_data.roles.private.get(REMOVE_ROLE_ID) {
        if ctx
            .author()
            .has_role(
                &ctx.http(),
                ctx.data().config_data.guild.main.guild_id,
                *role_id,
            )
            .await
            .unwrap_or(false)
        {
            return Ok(true);
        }
    }
    ctx.say("You don't have the proper permissions to use the enroll command")
        .await?;
    Ok(false)
}
