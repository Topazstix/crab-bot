use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{
    CommandDataOption,
    CommandDataOptionValue,
};


// private function to unwrap the data option
fn unwrap_data_option(option_value: &CommandDataOptionValue) -> Option<String> {
    
    // if option string => string,
    // if option boolean => Yes/No
    // else None
    if let CommandDataOptionValue::String(value) = option_value {
        Some(value.clone())
    } else if let CommandDataOptionValue::Boolean(value) = option_value {
        match value {
            true => Some("Yes".to_string()),
            false => Some("No".to_string()),
        }
    } else {
        None
    }

}//end unwrap_data_option


// public function to run the command
pub fn run(options: &[CommandDataOption]) -> String {

    // unwrap the data options from users responses
    let name_option = unwrap_data_option(options
        .get(0)
        .expect("Expected name option")
        .resolved
        .as_ref()
        .expect("Expected name object")).unwrap();

    let email_option = unwrap_data_option(options
        .get(1)
        .expect("Expected email option")
        .resolved
        .as_ref()
        .expect("Expected email object")).unwrap();

    let interests_option = unwrap_data_option(options
        .get(2)
        .expect("Expected interests option")
        .resolved
        .as_ref()
        .expect("Expected interests object")).unwrap();

    let add_to_email_option = unwrap_data_option(options
        .get(3)
        .expect("Expected add_to_email_distro option")
        .resolved
        .as_ref()
        .expect("Expected add_to_email_distro object")).unwrap();

    // return the formatted string
    format!(
        "Enrolling new student:\nName: {:?}\nEmail: {:?}\nInterests: {:?}\nAdd to email distro: {:?}",
        name_option, email_option, interests_option, add_to_email_option
    )

}//end run


// public function to register the command
pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    
    // create the enrollment command
    command
        .name("enrollment")
        .description("Enrollment commands")
        
        // create name sub option
        .create_option(|student_name| {
            student_name
                .name("name")
                .description("First name and last initial")
                .kind(CommandOptionType::String)
                .required(true)
        })
        
        // create email sub option
        .create_option(|student_email| {
            student_email
                .name("email")
                .description("School email address")
                .kind(CommandOptionType::String)
                .required(true)
        })
        
        // create interests sub option
        .create_option(|student_interests| {
            student_interests
                .name("interests")
                .description("Areas of interest.")
                .kind(CommandOptionType::String)
                .required(true)
        })
        
        // create email distro sub option
        .create_option(|student_distro| {
            student_distro
                .name("add_to_email_distro")
                .description("Add to email distro?")
                .kind(CommandOptionType::Boolean)
                .required(true)
        })

}//end enrollment