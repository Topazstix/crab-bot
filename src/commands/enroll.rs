include!(concat!(env!("OUT_DIR"), "/generated_roles.rs"));
use crate::checks::enroll_channel::enroll_channel;
use crate::checks::remove_role::remove_role;
use crate::storage::save_user::save_to_json;
use crate::storage::user::User;
use crate::utils::config::{ADMIN_ROLE_ID, REMOVE_ROLE_ID};
use crate::{Context, Error};
use poise::serenity_prelude::{EditMember, Mentionable, RoleId};
use regex::Regex;
use serde_json::json;

#[poise::command(
    slash_command,
    guild_only,
    check = "remove_role",
    check = "enroll_channel"
)]
pub async fn enroll(
    ctx: Context<'_>,
    #[description = "First and at least last initial"] name: String,
    #[description = "Your student email"] email: String,
    #[description = "Why are you interested in cyber club"] interests: String,
    #[description = "What role best describes you"] role: RoleEnum,
    #[description = "Would you like to occasionally receive emails"] email_distro: Option<bool>,
) -> Result<(), Error> {
    // Ensure the name input contains more than one word
    if name.split_whitespace().count() == 0 {
        ctx.reply("Need a last initial included").await?;
        return Ok(());
    }
    let email_regex = Regex::new("^[\\w\\-.]+@([\\w-]+\\.)+[\\w-]{2,}$").unwrap();
    if !email_regex.is_match(&email) {
        ctx.reply("Invalid email format").await?;
        return Ok(());
    }

    // Split name into first name and last initial
    let first = match name.split_ascii_whitespace().next() {
        Some(first_name) => first_name,
        None => {
            ctx.reply("invalid name must include last initial").await?;
            return Ok(());
        }
    };
    let last_initial = match name.split_ascii_whitespace().nth(1) {
        Some(second_word) => match second_word.chars().next() {
            Some(initial) => initial,
            None => {
                ctx.reply("invalid name must include last initial").await?;
                return Ok(());
            }
        },
        None => {
            ctx.reply("invalid name must include last initial").await?;
            return Ok(());
        }
    };
    let email_distro = email_distro.unwrap_or_default();

    // Check if the university name exists in the public roles
    if !ctx
        .data()
        .config_data
        .roles
        .public
        .contains_key(&role.to_string())
    {
        ctx.reply("Unknown university selected please try again")
            .await?;
        return Ok(());
    }
    let uni_role = ctx.data().config_data.roles.public.get(&role.to_string());
    if uni_role.is_none() {
        ctx.reply("Invalid Role").await?;
        return Ok(());
    }
    let uni_role = *uni_role.unwrap();

    // Retrieve some info about the guild member
    let guild_id = ctx.guild_id();
    let http = ctx.http();
    let member_id = ctx.author().id;
    let remove_role_id = ctx.data().config_data.roles.private.get(REMOVE_ROLE_ID);

    // Formulate an error output in case anything happens
    let error_format = match ctx.author_member().await {
        Some(member) => match guild_id {
            Some(guild) => match guild.roles(&ctx.http()).await {
                Ok(roles) => match ctx.data().config_data.roles.private.get(ADMIN_ROLE_ID) {
                    Some(admin_role_id) => match roles.get(&RoleId::new(*admin_role_id)) {
                        Some(role) => format!(
                            "Hi {}, Something has gone wrong. The people with {} will help you! Just message them!",
                            member.user.mention(),
                            role.mention()
                        ),
                        None => "An error occurred: Admin role not found.".to_string(),
                    },
                    None => "An error occurred: Admin role ID not found.".to_string(),
                },
                Err(_) => "An error occurred: Failed to fetch roles.".to_string(),
            },
            None => "An error occurred: Guild ID not found.".to_string(),
        },
        None => "An error occurred: Failed to fetch member.".to_string(),
    };

    // Try to assign new roles and nickname to the member
    match guild_id {
        Some(id) => {
            // Prepare the modification of the member
            let builder = EditMember::new()
                .roles(vec![uni_role])
                .nickname(format!("{} {}", first, last_initial));
            // Try to apply the changes
            match id.edit_member(&http, member_id, builder).await {
                Ok(member) => {
                    // Remove the remove_role if it exists
                    if let Some(role_id) = remove_role_id {
                        match member.remove_role(&http, *role_id).await {
                            Ok(_) => (),
                            Err(e) => {
                                // Handle errors by sending a message
                                println!("error removing role error: {}", e.to_string());
                                ctx.defer_ephemeral().await?;
                                ctx.reply(error_format).await?;
                                return Ok(());
                            }
                        }
                    }
                }
                Err(e) => {
                    // Handle errors by sending a message
                    println!("error editing member error: {}", e.to_string());
                    ctx.defer_ephemeral().await?;
                    ctx.reply(error_format).await?;
                    return Ok(());
                }
            }
        }
        None => {
            ctx.reply(error_format).await?;
            return Ok(());
        }
    };

    // Everything went fine, let's notify the user
    ctx.reply(format!("You have registered as:\nName: {} {}\nEmail: {}\nInterests: {}\nUniversity: {}\nAdd to Email Distro: {}", first, last_initial, email, interests, role, email_distro)).await?;

    // Prepare the user data as JSON
    let user_data_json = json!({
        "user_id": ctx.author().id.get(),
        "user_name": ctx.author().name,
        "name": format!("{} {}", first, last_initial),
        "role": role,
        "email": email,
        "interests": interests,
        "email_distro": email_distro,
        "points": 0,
        "thm_username": ""
    });

    // Convert the JSON to the User struct
    let user_data: User = serde_json::from_value(user_data_json).unwrap();

    // Try to save the user data on a JSON file
    if save_to_json(&user_data).is_err() {
        // Log any errors that happened during saving
        println!("Had an error while saving user data");
        ctx.reply(error_format).await?;
    }
    Ok(())
}
