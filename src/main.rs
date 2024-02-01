mod commands;
mod backend;


use std::collections::HashMap;
use regex::Regex;
use tracing::error;
use serde_json::json;
use serenity::prelude::*;
use serenity::async_trait;
use serenity::model::gateway::Ready;
use serenity::model::channel::Message;
use backend::database_storage::Enrollment;
use serenity::model::id::{ChannelId, GuildId};
use serenity::model::application::interaction::{Interaction, InteractionResponseType};
use serde_derive::Deserialize;
use toml::de::Error;

struct Bot;

#[derive(Debug, Deserialize)]
struct Data {
    token: HashMap<String, String>,
    guild: HashMap<String, u64>,
    roles: HashMap<String, u64>,
    channels: HashMap<String, u64>,
}

// takes in a mutable iterator of string literals '
// returns the user response
fn parse_response<'a>(responses: &mut dyn Iterator<Item = &'a str>) -> &'a str {
    // gets next value
    responses.next().expect("No Next value in registration response")
        // get the user input of the field
        .split(": ").nth(1).expect("No value associated with registration response")
        // trims quotations
        .trim_matches('"')
}

fn get_config() -> Result<Data, Error> {
   let data = std::fs::read_to_string("config.toml").unwrap();
    toml::from_str(&data)
}

fn get_roles(config: &Data) -> Vec<String> {
    config.roles.keys()
        .filter(|key| key.as_str() != REMOVE_ROLE_ID)
        .map(|value| value.to_owned() )
        .collect::<Vec<String>>()
}

// const env vars
const DESTIN_CHANNEL_ID: &str = "DESTIN_CHANNEL_ID";
const READING_CHANNEL_ID: &str = "READING_CHANNEL_ID";
const ENROLL_CHANNEL_ID: &str = "ENROLL_CHANNEL_ID";
const GUILD_ID: &str = "GUILD_ID";
const DISCORD_TOKEN: &str = "discord_token";
const REMOVE_ROLE_ID: &str = "REMOVE_ROLE_ID";

#[async_trait]
impl EventHandler for Bot {
    // handle reading and sending messages
    async fn message(&self, ctx: Context, msg: Message) {

        // secret command :)
        if msg.content == "!hello" {
            if let Err(e) = msg.channel_id.say(&ctx.http, "world!").await {
                error!("Error sending message: {:?}", e);
            };
            return
        }

        let config = get_config().expect("using config.toml gave an error");

        // pull Channel Id environment vars from .env file
        let destin_channel = ChannelId(*config.channels.get(DESTIN_CHANNEL_ID).expect("destin channel id not found"));
        let reading_channel = ChannelId(*config.channels.get(READING_CHANNEL_ID).expect("reading channel id not found"));
        let enroll_channel = ChannelId(*config.channels.get(ENROLL_CHANNEL_ID).expect("enroll channel id not found"));


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
            let roles = get_roles(&config);
            let remove_id = *config.roles.get(REMOVE_ROLE_ID).expect("Unable to find remove_id in config.toml");

            // Pull student responses from enrollment message
            // skips the first element in response iterator
            let mut response = msg.content.lines().skip(1);
            let nickname = parse_response(&mut response);
            let email_response = parse_response(&mut response);
            let interests_response = parse_response(&mut response);
            let uni_response = parse_response(&mut response);
            let distro_response = parse_response(&mut response);

            // remove entry point role if uni_response matches "uni_one" or "uni_two"
            for role in roles {
                if role == uni_response {
                    if let Err(e) = guild_id.member(&ctx.http, user_id).await.unwrap().remove_role(&ctx.http, remove_id).await {
                        error!("Error removing role: {:?}", e);
                    }
                    if let Err(e) = guild_id.member(&ctx.http, user_id).await.unwrap().add_role(&ctx.http, *config.roles.get(&role).expect("Unable to find role")).await {
                        error!("Error adding role: {:?}", e);
                    }
                    break
                }
            }

            // change the user's nickname for the guild to their response to the enrollment form
            if let Ok(member) = guild_id.member(&ctx.http, user_id).await {
                if let Err(e) = member.edit(&ctx.http, |guild_user| guild_user.nickname(nickname)).await {
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
        let config = get_config().expect("using config.toml gave an error");
        let guild_id = GuildId(*config.guild.get(GUILD_ID).expect("Unable to find GUILD_ID in config.toml"));

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
    let config = get_config().expect("using config.toml gave an error");
    let token: &String = config.token.get(DISCORD_TOKEN).expect("Unable to find DISCORD_TOKEN in config.toml");

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT |
        GatewayIntents::GUILD_MEMBERS | GatewayIntents::GUILDS;

    // Build our client.
    let mut client = Client::builder(token, intents)
        .event_handler(Bot)
        .await
        .expect("Error creating client");

    // Finally, start a single shard, and start listening to events.
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }

}//end main