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



struct Bot;

#[async_trait]
impl EventHandler for Bot {
    
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

    }//end interaction_create


    // handle bot ready event
    async fn ready(&self, ctx: Context, ready: Ready) {
        
        println!("{} is connected!", ready.user.name);

        // pull guild ID (discord server ID)
        let guild_id = GuildId(
            dotenv::var("GUILD_ID")
                .expect("Expected GUILD_ID in environment")
                .parse::<u64>()
                .expect("GUILD_ID must be an integer"),
        );

        // create commands for given guild
        let commands = GuildId::set_application_commands(&guild_id, &ctx.http, |commands| {
            commands
                .create_application_command(|command| commands::enrollment::register(command))
        }).await;

        println!("The following commands are available: {:#?}", commands);

    }//end ready


    // handle reading and sending messages
    async fn message(&self, ctx: Context, msg: Message) {

        // pull Channel Id enviornment vars from .env file
        let destin_channel = ChannelId(
            dotenv::var("DESTIN_CHANNEL_ID")
                .unwrap()
                .parse::<u64>()
                .expect("Destin ID Var not found")
        );
        let reading_channel = ChannelId(
            dotenv::var("READING_CHANNEL_ID")
                .unwrap()
                .parse::<u64>()
                .expect("Reading ID Var not found"),
        );
        let enroll_channel = ChannelId(
            dotenv::var("ENROLL_CHANNEL_ID")
                .unwrap()
                .parse::<u64>()
                .expect("Enroll ID Var not found"),
        );

        // secret command :)
        if msg.content == "!hello" {
            if let Err(e) = msg.channel_id.say(&ctx.http, "world!").await {
                error!("Error sending message: {:?}", e);
            }
        }

        // set regular expression patterns for matching messages
        let http_match = Regex::new(r"^(https|http|\^\^).*").unwrap();
        let enroll_match = Regex::new(r"^Enrolling new student:").unwrap();

        // if the message is in the reading channel and matches the http regex, 
        // send it to the destin channel
        if msg.channel_id == reading_channel {
            if http_match.is_match(&msg.content) && !msg.author.bot {

                // Make a prettier message with the original authors name to send
                let message = format!(".\n*This was originally posted by `{}`:*\n{}", msg.author.name, msg.content);

                if let Err(e) = destin_channel.say(&ctx.http, message).await {
                    error!("Error sending message: {:?}", e);
                }
            }
        }//end http match and send


        // if the message is in the enrollment channel and matches the enrollment regex,
        // add the student role, remove the entry point role, and change the user's nickname
        if msg.channel_id == enroll_channel {

            if enroll_match.is_match(&msg.content) && msg.author.bot {

                // pull user and guild IDs from the message
                let user_id = msg.interaction.as_ref().unwrap().user.id;
                let user_name = msg.interaction.unwrap().user.name;
                let guild_id = msg.guild_id.unwrap();

                // pull env vars
                let uni_one_id = RoleId(
                    dotenv::var("uni_one_ID")
                        .unwrap()
                        .parse::<u64>()
                        .expect("Student Role ID Var not found"),
                );
                let uni_two_id = RoleId(
                    dotenv::var("uni_two_ID")
                        .unwrap()
                        .parse::<u64>()
                        .expect("Student Role ID Var not found"),
                );
                let remove_id = RoleId(
                    dotenv::var("REMOVE_ROLE_ID")
                        .unwrap()
                        .parse::<u64>()
                        .expect("Remove Role ID Var not found"),
                );

                // Pull student responses from enrollment message
                let nickname = msg.content
                    .split("\n")
                    .collect::<Vec<&str>>()[1]
                    .split(": ")
                    .collect::<Vec<&str>>()[1]
                    .trim_matches('"')
                    .to_string();
                let email_response = msg.content
                    .split("\n")
                    .collect::<Vec<&str>>()[2]
                    .split(": ")
                    .collect::<Vec<&str>>()[1]
                    .trim_matches('"');
                let interests_response = msg.content
                    .split("\n")
                    .collect::<Vec<&str>>()[3]
                    .split(": ")
                    .collect::<Vec<&str>>()[1]
                    .trim_matches('"');
                let uni_response = msg.content
                    .split("\n")
                    .collect::<Vec<&str>>()[4]
                    .split(": ")
                    .collect::<Vec<&str>>()[1]
                    .trim_matches('"');
                let distro_response = msg.content
                    .split("\n")
                    .collect::<Vec<&str>>()[5]
                    .split(": ")
                    .collect::<Vec<&str>>()[1]
                    .trim_matches('"');

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

            }
        }//end enrollment match and send

    }//end message

}//end eventhandler for bot



#[tokio::main]
async fn main() {

    // Configure the client with your Discord bot token in the environment.
    let token = dotenv::var("DISCORD_TOKEN").expect("Expected a token in the environment");

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