mod commands;
pub use commands::Command;
mod state;
pub use state::State;
mod callback;
pub use callback::MyCallback;
mod text_commands;
pub use text_commands::TextCommand;
mod router;
use anyhow::Result;
use clap::Parser;
use teloxide::{
    dispatching::dialogue::InMemStorage,
    dptree::deps,
    payloads::DeleteWebhookSetters,
    prelude::{Dialogue, Dispatcher, LoggingErrorHandler, Requester},
    types::ChatId,
    utils::command::BotCommands,
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
    tgchannel: String,
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
    let tg_channel = format!("-{}", cli.tgchannel);
    let channel_id = ChatId(tg_channel.parse()?);
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

    // bot
    Dispatcher::builder(bot, router::master())
        .dependencies(deps![InMemStorage::<State>::new(), rpc_client, channel_id])
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
