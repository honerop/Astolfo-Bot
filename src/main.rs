use regex::Regex;
use reqwest::Client;
use serde::Deserialize;
use serenity::all::CreateAttachment;
use serenity::all::CreateMessage;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use sqlx::Row;
use sqlx::SqlitePool;
use sqlx::prelude::FromRow;
use sqlx::sqlite::SqliteError;
use sqlx::sqlite::SqliteQueryResult;
use sqlx::sqlite::SqliteRow;
use std::env;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::AsyncReadExt;
mod deepseek;

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

struct ArgsUse {
    page: Option<u32>,
    limit: Option<u32>,
    tags: Option<Vec<String>>,
}
struct DatabaseKey;

impl TypeMapKey for DatabaseKey {
    type Value = Arc<SqlitePool>;
}
fn handle_femboy_name(inp: u8) -> String {
    match inp {
        0..10 => String::from("Astolfo"),
        40..50 => String::from("Felix"),
        50..60 => String::from("Robin"),
        60..70 => String::from("Astolfo"),
        70..80 => String::from("Astolfo"),
        90..100 => String::from("Astolfo"),
        _ => String::from("m"),
    }
}
fn choose_image_based_on_name(name: &str) -> Option<PathBuf> {
    match name {
        "Astolfo" => Some(PathBuf::from("")),
        "Felix" => Some(PathBuf::from("")),
        "Robin" => Some(PathBuf::from("")),
        _ => Some(PathBuf::from("")),
    }
}

struct FemboyRarity {
    ratio: f32,
    rarity: String,
}
fn handle_femboy_rarity(inp: u8) -> FemboyRarity {
    match inp {
        0..60 => FemboyRarity {
            ratio: 1.0,
            rarity: String::from("Bronze"),
        },
        60..98 => FemboyRarity {
            ratio: 1.2,
            rarity: String::from("Silver"),
        },
        98..100 => FemboyRarity {
            ratio: 1.3,
            rarity: String::from("Silver"),
        },
        _ => FemboyRarity {
            ratio: 0.0,
            rarity: String::from("None"),
        },
    }
}

fn handle_combat_power(inp: &str) -> u16 {
    match inp {
        "Felix" => 600,
        "Astolfo" => 700,
        _ => 0,
    }
}

fn handle_health(inp: &str) -> u16 {
    match inp {
        "Felix" => 4500,
        "Astolfo" => 4000,
        _ => 0,
    }
}
fn handle_unique_attack(inp: &str) -> String {
    match inp {
        "Felix" => String::from("UwU"),
        _ => String::new(),
    }
}
#[derive(Debug, FromRow)]
struct Item {
    id: i64,
    user_id: i64,
    rarity: Option<String>,
    femboy_name: String,
}
async fn insert_new_femboy(
    pool: &SqlitePool,
    user_id: i64,
    femboy_rarity: &str,
    femboy_name: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO item (user_id,femboy_name,rarity,femboy_name) VALUES (?,?,?,?) ")
        .bind(&(user_id))
        .bind(femboy_name)
        .bind(femboy_rarity)
        .bind("Astolfo")
        .execute(&*pool)
        .await?;
    Ok(())
}

async fn get_item_from_user_id(pool: &SqlitePool, user_id: i64) -> Result<Vec<Item>, sqlx::Error> {
    let items = sqlx::query_as::<_, Item>(
        "SELECT id, user_id, rarity, femboy_name FROM item WHERE user_id = ?",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(items)
}
async fn remove_money_from_user_id(
    pool: &SqlitePool,
    user_id: &i64,
    amount: &u32,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE users SET money = money - ? WHERE id = ? ")
        .bind(amount)
        .bind(user_id)
        .execute(&*pool)
        .await?;
    Ok(())
}
async fn send_message_wrapper(msg: &Message, ctx: &Context, inp: &str) {
    match msg.channel_id.say(&ctx.http, inp).await {
        Ok(_) => {}
        Err(e) => { //TODO handle unsuccesful sending of data}
        }
    }
}

async fn handle_femboycoin(pool: &SqlitePool, arg: &str, msg: &Message, ctx: &Context) {
    if arg.trim() == "balance" {
        match sqlx::query("SELECT * FROM users WHERE id=?")
            .bind(msg.author.id.get() as i64)
            .fetch_optional(pool)
            .await
        {
            Ok(None) => {
                send_message_wrapper(msg, ctx, "You are not registered in database").await;
            }
            Ok(Some(row)) => {
                let money: i32 = row.get("money");
                let message = format!("Your current money: {}", money);
                msg.channel_id.say(&ctx.http, message).await;
            }
            Err(e) => {
                msg.channel_id.say(&ctx.http, format!("Error: {e}")).await;
            }
        }
    }

    if arg.trim() == "roll" {
        //TODO make rand internal in function
        let femboy_name = handle_femboy_name(rand::random_range(0..=100));
        let femboy_rarity = handle_femboy_rarity(rand::random_range(0..=100));
        let health = (handle_health(&femboy_name) as f32 * femboy_rarity.ratio) as i64;
        let combat_power = (handle_combat_power(&femboy_name) as f32 * femboy_rarity.ratio) as i64;

        //TODO make this more safe, when couldnt insert new femboy then do nothing
        {
            insert_new_femboy(
                pool,
                msg.author.id.get() as i64,
                &femboy_rarity.rarity,
                &femboy_name,
            )
            .await;
            remove_money_from_user_id(pool, &(msg.author.id.get() as i64), &2).await;
        }
        msg.channel_id
            .say(
                &ctx.http,
                format!(
                    "You got: {} of rarity: {} ,with health: {},  power combat {},unique attacks: {}",
                    &femboy_name, femboy_rarity.rarity, health, combat_power, ""
                ),
            )
            .await;
        let image_path = match choose_image_based_on_name(&femboy_name) {
            Some(v) => v,
            None => return,
        };
        let image_name = match image_path.file_name() {
            Some(v) => v,
            None => {
                send_message_wrapper(msg, ctx, "Couldnt send because file has no name").await;
                return;
            }
        };

        let mut file = tokio::fs::File::open(Path::new(&image_path)).await.unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await;

        let attachment = CreateAttachment::bytes(buffer, image_name.to_string_lossy().to_string());

        msg.channel_id
            .send_files(&ctx.http, vec![attachment], CreateMessage::new())
            .await;
    }

    if arg.trim() == "register" {
        match sqlx::query("INSERT INTO users (id,money) VALUES (?,?)")
            .bind(msg.author.id.get() as i64)
            .bind(10 as i32)
            .execute(&*pool)
            .await
        {
            Ok(_) => {
                msg.channel_id
                    .say(&ctx.http, "You have been registered")
                    .await;
            }
            Err(e) => {
                msg.channel_id
                    .say(&ctx.http, "You are already registered")
                    .await;
            }
        }
    }
    if arg.trim() == "inventory" {
        match sqlx::query("SELECT * from users WHERE id=?")
            .bind(msg.author.id.get() as i64)
            .execute(&*pool)
            .await
        {
            Ok(v) => {
                let result_from_db = get_item_from_user_id(&*pool, msg.author.id.get() as i64)
                    .await
                    .unwrap();
                let result: Vec<String> = result_from_db
                    .iter()
                    .map(|x| format!("{} o id: {}", &x.femboy_name, x.id))
                    .collect();
                msg.channel_id
                    .say(&ctx.http, format!("Current inventory: {:?}", result))
                    .await
                    .unwrap();
            }
            Err(e) => {
                msg.channel_id
                    .say(&ctx.http, format!("cos nie pyklo: {e}"))
                    .await
                    .unwrap();
            }
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if let Some(idx) = msg.content.find("astolfo") {
            let prompt = msg.content[0..idx].to_string()
                + &msg.content[idx + "astolfo".len()..msg.content.len()];
        } else if msg.content.starts_with("!r34") {
            let args = msg.content[5..].split(' ');
            let mut colected = ArgsUse {
                page: None,
                limit: None,
                tags: None,
            };
            for arg in args {
                if arg.starts_with("page=") {
                    let page: u32 = match arg[5..arg.len()].parse() {
                        Ok(v) => v,
                        Err(_) => return,
                    };
                    colected.page = Some(page)
                } else if arg.starts_with("limit=") {
                    let limit: u32 = match arg[6..arg.len()].parse() {
                        Ok(v) => v,
                        Err(_) => return,
                    };
                    colected.limit = Some(limit)
                } else if arg.starts_with("tags=") {
                    let limit: Vec<String> = arg[5..arg.len()]
                        .split(',')
                        .map(|v| v.to_string())
                        .collect();
                    colected.tags = Some(limit)
                }
            }

            let page = colected.page.unwrap_or(0);
            let limit = colected.limit.unwrap_or(10);
            let tags = colected.tags.unwrap_or(vec![]);
            let joined = tags.join("+");
            let url = format!(
                "https://safebooru.org/index.php?&api_key=&user_id=5331151&page=dapi&s=post&q=index&pid={}&limit={}&tags={}",
                page, limit, joined
            );

            let client = Client::new();

            let Ok(resp) = client.get(&url).send().await else {
                let _ = msg
                    .channel_id
                    .say(&ctx.http, "❌ Failed to fetch data.")
                    .await;
                return;
            };

            let Ok(text) = resp.text().await else {
                let _ = msg
                    .channel_id
                    .say(&ctx.http, "❌ Failed to read response.")
                    .await;
                return;
            };

            let re = Regex::new(r#"file_url="([^"]+)""#).unwrap();
            let file_urls: Vec<&str> = re
                .captures_iter(&text)
                .filter_map(|cap| cap.get(1))
                .map(|m| m.as_str())
                .collect();

            if file_urls.is_empty() {
                let _ = msg.channel_id.say(&ctx.http, "❌ Bedzie robione.").await;
                return;
            }

            let response_text = file_urls.join("\n");

            if let Err(why) = msg.channel_id.say(&ctx.http, response_text).await {
                println!("Error sending message: {why:?}");
            }
        } else if msg.content.starts_with("!femboycoin") {
            let data = ctx.data.read().await;
            let pool = data.get::<DatabaseKey>().cloned().unwrap();
            let arg = &msg.content[12..msg.content.len()];
            handle_femboycoin(&*pool, arg, &msg, &ctx).await;
        } else if msg.content.starts_with("(femboycoin") {
            let data = ctx.data.read().await;
            let pool = data.get::<DatabaseKey>().cloned().unwrap();
            let arg = &msg.content[12..msg.content.len()];
            handle_femboycoin(&*pool, arg, &msg, &ctx).await;
        }
    }
}

#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("ASTOLFO").unwrap();
    let env = env::var("DATABASE_URL").unwrap();

    let pool = sqlx::SqlitePool::connect("sqlite:mydb.db").await.unwrap();

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MEMBERS;

    // Create a new instance of the Client, logging in as a bot. This will automatically prepend
    // your bot token with "Bot ", which is a requirement by Discord for bot users.
    let mut client = serenity::Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");
    {
        let mut data = client.data.write().await;
        data.insert::<DatabaseKey>(Arc::new(pool));
    }

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform exponential backoff until
    // it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
