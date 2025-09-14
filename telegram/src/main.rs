mod commands;
pub use commands::Command;
mod state;
use publisher::Publisher;
use shared::models::{Post, Status};
pub use state::State;
mod callback;
pub use callback::MyCallback;
mod text_commands;
pub use text_commands::TextCommand;
mod router;
use anyhow::Result;
use clap::Parser;
use teloxide::{
    dispatching::dialogue::InMemStorage, dptree::deps, payloads::DeleteWebhookSetters, prelude::*,
    types::InputFile, utils::command::BotCommands,
};

#[derive(Parser)]
#[command(name = "telegram bot for SMMaster", version, about = "SMM telegram bot", long_about = None)]
struct Cli {
    /// Define port to serve
    #[arg(short, long)]
    port: Option<u16>,
    /// Bearer token
    #[arg(short, long)]
    bearer: Option<String>,
    /// Telegram bot token
    #[arg(long)]
    tgtoken: String,
    /// Telegram channel id
    #[arg(long)]
    tgchannel: i64,
    /// VK access token
    #[arg(long)]
    vktoken: String,
    /// VK group id
    #[arg(long)]
    vkgroup: String,
}

pub type MyDialogue = Dialogue<State, InMemStorage<State>>;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().init();

    // config
    let cli = Cli::parse();
    let port = cli.port.unwrap_or(50052);
    let bearer = cli.bearer.unwrap_or("some-secret-token".into());
    let tg_token = cli.tgtoken;
    let tg_channel = cli.tgchannel * -1;
    let vk_token = cli.vktoken;
    let vk_group = cli.vkgroup;

    // rpc
    let rpc_client = client::Client::new(port, bearer).await?;

    // tg
    let bot = teloxide::Bot::new(tg_token);
    bot.delete_webhook().drop_pending_updates(true).await?;
    bot.set_my_commands(Command::bot_commands()).await?;
    tracing::info!("🚀 Starting 🤖  bot");

    // vk
    let _ = (vk_token, vk_group);

    // publisher

    let publisher = Publisher::new(bot.clone(), tg_channel, rpc_client.clone());
    tokio::spawn(publisher.run());

    // bot
    Dispatcher::builder(bot, router::master())
        .dependencies(deps![InMemStorage::<State>::new(), rpc_client])
        .default_handler(|upd| async move {
            tracing::warn!("Unhandled update: {upd:?}");
        })
        .error_handler(LoggingErrorHandler::with_custom_text(
            "An error has occurred in the dispatcher",
        ))
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
    Ok(())
}

pub fn moscow(dt: chrono::DateTime<chrono::Utc>) -> String {
    let moscow_offset = chrono::FixedOffset::east_opt(3 * 60 * 60).unwrap();
    let moscow_dt = dt.with_timezone(&moscow_offset);
    let string = moscow_dt.format("%d-%m-%Y %H:%M:%S");
    format!("{string}")
}
pub fn to_utc(string: &str) -> Result<chrono::DateTime<chrono::Utc>> {
    use chrono::TimeZone;
    let naive_dt = chrono::NaiveDateTime::parse_from_str(string, "%Y-%m-%d %H:%M")?;
    let moscow_offset = chrono::FixedOffset::east_opt(3 * 60 * 60).unwrap();
    let moscow_dt = moscow_offset.from_local_datetime(&naive_dt).unwrap();
    Ok(moscow_dt.with_timezone(&chrono::Utc))
}

pub async fn send_post(bot: &Bot, msg: &Message, post: &Post) -> Result<()> {
    let text = match post.status {
        shared::models::Status::Pending => {
            format!(
                "<b>{title}</b>\n{content}\nОпубликую: <code>{date}</code>",
                title = post.title,
                content = post.content,
                date = moscow(post.publish_datetime.unwrap_or_default()),
            )
        }
        shared::models::Status::Published => {
            format!(
                "<b>{title}</b>\n{content}\nОпубликован: <code>{date}</code>",
                title = post.title,
                content = post.content,
                date = moscow(post.publish_datetime.unwrap_or_default()),
            )
        }
        _ => {
            format!(
                "<b>{title}</b>\n{content}",
                title = post.title,
                content = post.content
            )
        }
    };
    let mu = if post.status == Status::Published {
        MyCallback::published_kb(post.id)
    } else {
        MyCallback::not_published_kb(post.id)
    };
    if let Some(p) = post.tg_photo_file_id.as_ref() {
        let photo = InputFile::file_id(p.to_string().into());
        bot.send_photo(msg.chat.id, photo)
            .caption(text)
            .reply_markup(mu)
            .parse_mode(teloxide::types::ParseMode::Html)
            .await?;
    } else if let Some(v) = post.tg_video_file_id.as_ref() {
        let video = InputFile::file_id(v.to_string().into());
        bot.send_video(msg.chat.id, video)
            .caption(text)
            .reply_markup(mu)
            .parse_mode(teloxide::types::ParseMode::Html)
            .await?;
    } else {
        bot.send_message(msg.chat.id, text)
            .reply_markup(mu)
            .parse_mode(teloxide::types::ParseMode::Html)
            .await?;
    }
    Ok(())
}
