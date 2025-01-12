use serde::{Deserialize, Serialize};
use std::collections::HashMap;
// constants

// token
pub const DISCORD_TOKEN: &str = "discord_token";
// roles.private
pub const REMOVE_ROLE_ID: &str = "REMOVE_ROLE_ID";
pub const ADMIN_ROLE_ID: &str = "ADMIN_ROLE_ID";
// roles.public
// channels
pub const ENROLL_CHANNEL: &str = "ENROLL_CHANNEL_ID";
pub const READING_CHANNEL: &str = "READING_CHANNEL_ID";
// pub const LOGGING_CHANNEL: &str = "LOGGING_CHANNEL_ID";
pub const MEETING_CHANNEL: &str = "MEETING_CHANNEL_ID";

// config file types
#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct ConfigData {
    #[serde(alias = "TOKEN")]
    pub token: HashMap<String, String>,
    #[serde(alias = "GUILD")]
    pub guild: Guild,
    #[serde(alias = "ROLES")]
    pub roles: Roles,
    #[serde(alias = "CHANNELS")]
    pub channels: HashMap<String, u64>,
    #[serde(alias = "RSS")]
    pub rss: Rss,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Roles {
    #[serde(alias = "PUBLIC")]
    pub public: HashMap<String, u64>,
    #[serde(alias = "PRIVATE")]
    pub private: HashMap<String, u64>,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Guild {
    #[serde(alias = "MAIN")]
    pub main: GuildMain,
    #[serde(alias = "PARTNERS")]
    pub partners: HashMap<String, GuildPartners>,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Rss {
    #[serde(alias = "FEED")]
    pub feed: HashMap<String, Feed>,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Feed {
    pub url: String,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct GuildMain {
    #[serde(alias = "GUILD_ID")]
    pub guild_id: u64,
    #[serde(alias = "PRESIDENT")]
    pub president: u64,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct GuildPartners {
    #[serde(alias = "ID")]
    pub id: u64,
    #[serde(alias = "NAME")]
    pub name: String,
    #[serde(alias = "SEND_NEWS")]
    pub send_news: bool,
    #[serde(alias = "NEWS_CHANNEL")]
    pub news_channel: u64,
}

// read and return result of the config file
pub fn get_config() -> Result<ConfigData, toml::de::Error> {
    let data = std::fs::read_to_string("config.toml").expect("Unable to find config.toml file");
    toml::from_str(&data)
}
