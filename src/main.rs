use std::env;
use regex::Regex;
use reqwest::{Method, header};
use lazy_static::lazy_static;
use reqwest::header::{HeaderMap, HeaderValue};

use serenity::{
    async_trait,
    client::bridge::gateway::GatewayIntents,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};

// Just use a really generic User-Agent header
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/94.0.4606.81 Safari/537.36";

// Hardcode the data in the form
const FORM_DATA: &str = "q1=No&q2=No&q3=No&q4=No&q5=No&q6=No&q7=No&q8=No&what=Submit";

lazy_static! {
    static ref HTTP_CLIENT: reqwest::Client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .expect("Error creating HTTP client");
}

async fn send_form(url: &str) -> Result<reqwest::Response, reqwest::Error> {
    let request = HTTP_CLIENT.request(Method::POST, url)
        .body(FORM_DATA)
        .headers({
            let mut map = HeaderMap::new();
            map.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/x-www-form-urlencoded")
            );
            map
        })
        .build()?;

    HTTP_CLIENT.execute(request).await
}


struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot { return }
        lazy_static! {
            static ref KEY_REGEX: Regex = Regex::new(r"checkin\.uwaterloo\.ca/campuscheckin/screen.php?").unwrap();
            static ref CONFIRM_REGEX: Regex = Regex::new(r"Thank you for").unwrap();
        }
        let url = &msg.content;
        println!("Received message from {}", msg.author.name);

        let mut reply = "Error completing form";
        if let Some(_) = KEY_REGEX.find(url) {
            if let Ok(resp) = send_form(url).await {
                if let Some(_) = CONFIRM_REGEX.find(&resp.text().await.unwrap()) {
                    reply = "Successfully completed form! Check your email.";
                    println!("Successfully completed form for {}", msg.author.name);
                }
            }
        } else {
            reply = "Invalid URL."
        }
        msg.reply(ctx, reply).await.expect("Error sending message");
    }

    async fn ready(&self, _ctx: Context, data_about_bot: Ready) {
        println!("{} is connected with id {}", data_about_bot.user.name, data_about_bot.user.id);
    }
}

#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    // Build our client.
    let mut client = Client::builder(token)
        .event_handler(Handler)
        .intents(GatewayIntents::DIRECT_MESSAGES)
        .await
        .expect("Error creating Discord client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}