mod commands;
mod backend;


use regex::Regex;
use tracing::error;
use serde_json::json;
use serenity::prelude::*;
use serenity::async_trait;
use serenity::model::gateway::Ready;
use serenity::model::channel::Message;
use backend::database_storage::Enrollment;
use serenity::model::id::{ChannelId, GuildId, RoleId};
use serenity::model::application::interaction::{Interaction, InteractionResponseType};
use std::ffi::OsStr;
use std::str::FromStr;

struct Bot;

fn parse_env<T, K: FromStr>(var: &T) -> K where T: AsRef<OsStr> + std::fmt::Debug + Sized  {
    dotenv::var(var).unwrap().parse().unwrap_or_else(|_| panic!("{:?} not found / not integer", var))
}

fn parse_response<'a>(responses: &mut dyn Iterator<Item=&str>) -> &'a str {
    responses.next().expect("No Next value in registration response").split(": ").nth(1).expect("No value associated with registration response").trim_matches('"')
}

// const env vars
const DESTIN_CHANNEL_ID: &str = "DESTIN_CHANNEL_ID";
const READING_CHANNEL_ID: &str = "READING_CHANNEL_ID";
const ENROLL_CHANNEL_ID: &str = "ENROLL_CHANNEL_ID";
const GUILD_ID: &str = "GUILD_ID";
const DISCORD_TOKEN: &str = "DISCORD_TOKEN";
const UNI_ONE_ID: &str = "uni_one_ID";
const UNI_TWO_ID: &str = "uni_two_ID";
const REMOVE_ROLE_ID: &str = "REMOVE_ROLE_ID";

#[async_trait]
impl EventHandler for Bot {
    // handle reading and sending messages
    async fn message(&self, ctx: Context, msg: Message) {

        // pull Channel Id environment vars from .env file
        let destin_channel = ChannelId(parse_env(&DESTIN_CHANNEL_ID));
        let reading_channel = ChannelId(parse_env(&READING_CHANNEL_ID));
        let enroll_channel = ChannelId(parse_env(&ENROLL_CHANNEL_ID));

        // secret command :)
        if msg.content == "!hello" {
            if let Err(e) = msg.channel_id.say(&ctx.http, "world!").await {
                error!("Error sending message: {:?}", e);
            };
            return
        }

        // set regular expression patterns for matching messages
        let http_match = Regex::new(r"^(https|http|\^\^).*").unwrap();
        let enroll_match = Regex::new(r"^Enrolling new student:").unwrap();

        // if the message is in the reading channel and matches the http regex,
        // send it to the destin channel
        if msg.channel_id == reading_channel && http_match.is_match(&msg.content) && !msg.author.bot {

            // Make a prettier message with the original authors name to send
            let message = format!(".\n*This was originally posted by `{}`:*\n{}", msg.author.name, msg.content);

            if let Err(e) = destin_channel.say(&ctx.http, message).await {
                error!("Error sending message: {:?}", e);
            }
        }//end http match and send


        // if the message is in the enrollment channel and matches the enrollment regex,
        // add the student role, remove the entry point role, and change the user's nickname
        if msg.channel_id == enroll_channel && enroll_match.is_match(&msg.content) && msg.author.bot {

            // pull user and guild IDs from the message
            let user_id = msg.interaction.as_ref().unwrap().user.id;
            let user_name = msg.interaction.unwrap().user.name;
            let guild_id = msg.guild_id.unwrap();

            // pull env vars
            let uni_one_id = RoleId(parse_env(&UNI_ONE_ID));
            let uni_two_id = RoleId(parse_env(&UNI_TWO_ID));
            let remove_id = RoleId(parse_env(&REMOVE_ROLE_ID));

            // Pull student responses from enrollment message
            let mut data = msg.content.lines().skip(1);
            let nickname = parse_response(&mut data);
            let email_response = parse_response(&mut data);
            let interests_response = parse_response(&mut data);
            let uni_response = parse_response(&mut data);
            let distro_response = parse_response(&mut data);

            // remove entry point role if uni_response matches "uni_one" or "uni_two"
            if let "uni_one" | "uni_two" = uni_response {
                if let Err(e) = guild_id.member(&ctx.http, user_id).await.unwrap().remove_role(&ctx.http, remove_id).await {
                    error!("Error removing role: {:?}", e);
                }

                // add university student roles
                if uni_response == "uni_one" {
                    if let Err(e) = guild_id.member(&ctx.http, user_id).await.unwrap().add_role(&ctx.http, uni_one_id).await {
                        error!("Error adding role: {:?}", e);
                    }
                } else if uni_response == "uni_two" {
                    if let Err(e) = guild_id.member(&ctx.http, user_id).await.unwrap().add_role(&ctx.http, uni_two_id).await {
                        error!("Error adding role: {:?}", e);
                    }
                }
            }

            // change the user's nickname for the guild to their response to the enrollment form
            if let Ok(member) = guild_id.member(&ctx.http, user_id).await {
                if let Err(e) = member.edit(&ctx.http, |guild_user| guild_user.nickname(&nickname)).await {
                    error!("Error changing nickname: {:?}", e);
                } else {
                    error!("Error: Member not found");
                }
            }

            // store the users data in the database
            let user_data_json = json!({
                "user_id": user_id.as_u64(),
                "user_name": user_name,
                "name": nickname,
                "university": uni_response,
                "email": email_response,
                "interests": interests_response,
                "email_distro": distro_response,
            });

            let user_data: Enrollment = serde_json::from_value(user_data_json).unwrap();

            if let Err(e) = backend::database_storage::save_to_json(&user_data) {
                error!("Error saving to json: {:?}", e);
            }

        }//end enrollment match and send

    }//end interaction_create


    // handle bot ready event
    async fn ready(&self, ctx: Context, ready: Ready) {
        
        println!("{} is connected!", ready.user.name);

        // pull guild ID (discord server ID)
        let guild_id = GuildId(parse_env(&GUILD_ID));

        // create commands for given guild
        let commands = GuildId::set_application_commands(&guild_id, &ctx.http, |commands| {
            commands
                .create_application_command(|command| commands::enrollment::register(command))
        }).await;

        println!("The following commands are available: {:#?}", commands);

    }//end ready


    // handle event interactions from server (slash commands)
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {

        if let Interaction::ApplicationCommand(command) = interaction {
            println!("Received command interaction: {:#?}", command);

            // determine what command to run from the users input
            let content = match command.data.name.as_str() {
                "enrollment" => commands::enrollment::run(&command.data.options),
                _ => "not implemented :(".to_string(), // handle invalid commands
            };

            // respond to the command with the content from the command
            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| message.content(content))
            }).await {
                error!("Cannot respond to slash command: {}", why);
            }

        }

    }//end message

}//end EventHandler for bot



#[tokio::main]
async fn main() {

    // Configure the client with your Discord bot token in the environment.
    let token: String = parse_env(&DISCORD_TOKEN);

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT |
        GatewayIntents::GUILD_MEMBERS | GatewayIntents::GUILDS;

    // Build our client.
    let mut client = Client::builder(&token, intents)
        .event_handler(Bot)
        .await
        .expect("Error creating client");

    // Finally, start a single shard, and start listening to events.
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }

}//end main