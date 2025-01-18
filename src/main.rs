use std::env;

use serenity::async_trait;
use serenity::json::json;
use serenity::model::channel::Message;
use serenity::prelude::*;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, _: Context, msg: Message) {
        if msg.author.bot {
            return;
        }
        let icon = msg.author.static_face();
        let message = msg.content;
        let username = msg.author.name;
        let content_part = reqwest::multipart::Part::text(
            json!(
                {
                    "username": username,
                    "content": message,
                    "avatar_url": icon,
                }
            )
            .to_string(),
        );
        let mut form = reqwest::multipart::Form::new().part("payload_json", content_part);

        for file in msg.attachments {
            let bin = file.download().await;
            if let Ok(bin) = bin {
                let file_part =
                    reqwest::multipart::Part::bytes(bin).file_name(file.filename.clone());
                form = form.part(file.id.to_string(), file_part);
            }
        }

        let client = reqwest::Client::new();
        let _ = client
            .post(env::var("WEBHOOK_URL").unwrap())
            .multipart(form)
            .send()
            .await;
    }
}

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
