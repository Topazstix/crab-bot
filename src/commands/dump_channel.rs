use std::fs::File;
use std::io::Write;
use poise::futures_util::StreamExt;
use poise::serenity_prelude::Channel;
use regex::Regex;
use crate::{Context, Error};

#[poise::command(slash_command, ephemeral)]
pub async fn dump(ctx: Context<'_>, channel: Channel) -> Result<(), Error>{
    ctx.say("Dumping channel").await?;
    let mut messages = channel.id().messages_iter(&ctx).boxed();
    let mut urls = vec![];
    while let Some(message) = messages.next().await.transpose()? {
        let website_regex = Regex::new("^https?://([\\w.-]+).[a-z]{3,4}(/.*)?$").unwrap();
        let content = message.content.clone();
        if website_regex.is_match(&content) {
            urls.push(content);

        }
    }
    println!("Dumped {} messages", urls.len());
    println!("{:?}", urls);
    let mut file = File::create("dump.out")?;

    for item in urls {
        writeln!(file, "{}", item)?;
    }

    Ok(())
}