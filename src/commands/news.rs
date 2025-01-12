use poise::CreateReply;
use poise::serenity_prelude::{ChannelId, Colour, CreateEmbed, CreateMessage};
use meta_fetcher::Metadata;
use rand::Rng;
use regex::Regex;
use crate::{Context, Error};


#[poise::command(slash_command)]
pub async fn news(
    ctx: Context<'_>,
    url: String,
) -> Result<(), Error> {
    let website_regex = Regex::new("^https?://([\\w.-]+).[a-z]{3,4}(/.*)?$").unwrap();
    if !website_regex.is_match(&url) {
        ctx.defer_ephemeral().await?;
        ctx.reply("Invalid URL").await?;
        return Ok(())
    }
    let channels_to_send_news_to = &ctx.data().config_data.guild.partners;
    // for some fun colors on the side
    let color = {
        let mut rng = rand::thread_rng();
         Colour::from_rgb(
            rng.gen_range(0..=255),
            rng.gen_range(0..=255),
            rng.gen_range(0..=255),
         )
    };


    let user = &ctx.author().name;

    let mut embed = CreateEmbed::default();
    embed = embed.url(url.clone());
    embed = embed.color(color);

    // get the metadata from the page to make it pretty 💅
    let meta = Metadata::from_url(&url)?;

    if let Some(title) = meta.title {
        embed = embed.title(title);
    } else {
        embed = embed.title("News Message");
    }
    // get the image
    if let Some(image) = meta.image {
        embed = embed.image(image);
    }
    let mut msg = CreateMessage::new();
    let mut own = CreateReply::default();
    if let Some(description) = meta.description {
        embed = embed.description(description);
        msg = msg.content(format!("Article provided by {}", user));
        own = own.content(format!("Article provided by {}", user));
    } else {
        embed = embed.description(format!("Article provided by {}", user));
    }
    own = own.embed(embed.clone());
    msg = msg.embed(embed);

    // send discord the reply so it doesn't freak out
    ctx.send(own).await?;
    for i in channels_to_send_news_to {
        let channel_id = ChannelId::from(i.1.news_channel);
        channel_id.send_message(ctx.http(), msg.clone()).await?;

    }
    Ok(())
}
