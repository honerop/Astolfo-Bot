use std::env;
use regex::Regex;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Posts {
    #[serde(rename = "post")]
    posts: Vec<Post>,
}

#[derive(Debug, Deserialize)]
struct Post {
    #[serde(rename = "file_url")]
    file_url: String,

    #[serde(rename = "tags")]
    tags: String,

    #[serde(rename = "id")]
    id: u32,
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "siema astolfo" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "siemano ʘ‿ʘ").await {
                println!("Error sending message: {why:?}");
            }
        } else if msg.content.starts_with("!r34 ") {
            let arg = msg.content[5..].trim();

            let page: u32 = match arg.parse() {
                Ok(n) => n,
                Err(_) => {
                    let _ = msg.channel_id.say(&ctx.http, "❌ Invalid number.").await;
                    return;
                }
            };

            let url = format!(

                "https://rule34.xxx/index.php?page=dapi&s=post&q=index&pid={}&limit=5&tags=astolfo",
                page
            );

            let client = Client::new();

            let Ok(resp) = client.get(&url).send().await else {
                let _ = msg.channel_id.say(&ctx.http, "❌ Failed to fetch data.").await;
                return;
            };

            let Ok(text) = resp.text().await else {
                let _ = msg.channel_id.say(&ctx.http, "❌ Failed to read response.").await;
                return;
            };

            let re = Regex::new(r#"file_url="([^"]+)""#).unwrap();
            let file_urls: Vec<&str> = re
                .captures_iter(&text)
                .filter_map(|cap| cap.get(1))
                .map(|m| m.as_str())
                .collect();

            if file_urls.is_empty() {
                let _ = msg.channel_id.say(&ctx.http, "❌ No results found.").await;
                return;
            }

            let response_text = file_urls.join("\n");

            if let Err(why) = msg.channel_id.say(&ctx.http, response_text).await {
                println!("Error sending message: {why:?}");
            }
        }
    }
}

#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("ASTOLFO").unwrap();
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT | GatewayIntents::GUILD_MEMBERS;

    // Create a new instance of the Client, logging in as a bot. This will automatically prepend
    // your bot token with "Bot ", which is a requirement by Discord for bot users.
    let mut client =
        serenity::Client::builder(&token, intents).event_handler(Handler).await.expect("Err creating client");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform exponential backoff until
    // it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
