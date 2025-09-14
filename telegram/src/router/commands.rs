use anyhow::{Result, anyhow};
use dptree::case;
use teloxide::{dispatching::DpHandlerDescription, prelude::*, utils::command::BotCommands};

use crate::{Command, TextCommand};

pub(super) fn router() -> Handler<'static, Result<()>, DpHandlerDescription> {
    teloxide::filter_command::<Command, _>()
        .branch(case![Command::Start].endpoint(start))
        .branch(case![Command::Help].endpoint(help))
}
async fn start(bot: Bot, msg: Message, mut rpc_client: client::Client) -> Result<()> {
    let from = msg.from.ok_or(anyhow!("no field 'from' on message"))?;
    let id = from.id.0.try_into()?;
    let has_admin = rpc_client.has_admin().await?;
    match rpc_client.get_user(id).await? {
        Some(existing) => {
            let name = if let Some(last) = existing.last_name {
                format!("{first} {last}", first = existing.first_name)
            } else {
                existing.first_name
            };
            match existing.role {
                shared::models::Role::Guest => {
                    let text = format!("Рад снова вас видеть, <b>{name}</b>!");
                    bot.send_message(msg.chat.id, text)
                        .parse_mode(teloxide::types::ParseMode::Html)
                        .reply_markup(TextCommand::guest_keyboard())
                        .await?;
                }
                shared::models::Role::Editor => {
                    let text =
                        format!("Рад снова вас видеть, <b>{name}</b>! Чем могу быть полезен?");
                    bot.send_message(msg.chat.id, text)
                        .parse_mode(teloxide::types::ParseMode::Html)
                        .reply_markup(TextCommand::editor_keyboard())
                        .await?;
                }
                shared::models::Role::Admin => {
                    let text = format!(
                        "Рад снова вас видеть, <b>Мастер {name}</b>! Чем могу быть вам полезен?"
                    );
                    bot.send_message(msg.chat.id, text)
                        .parse_mode(teloxide::types::ParseMode::Html)
                        .reply_markup(TextCommand::admin_keyboard())
                        .await?;
                }
            }
        }
        None => {
            let mut created = rpc_client
                .create_user(
                    id,
                    from.first_name,
                    from.last_name,
                    from.username,
                    from.language_code,
                )
                .await?
                .ok_or(anyhow!("error creating new user"))?;
            if !has_admin {
                created.role = shared::models::Role::Admin;
                let updated = rpc_client
                    .update_user(created)
                    .await?
                    .ok_or(anyhow!("error updating user role to admin"))?;
                let name = if let Some(last) = updated.last_name {
                    format!("{first} {last}", first = updated.first_name)
                } else {
                    updated.first_name
                };
                let text =
                    format!("Добро пожаловать, <b>{name}</b>! Вы теперь <i>администратор</i>.");
                bot.send_message(msg.chat.id, text)
                    .parse_mode(teloxide::types::ParseMode::Html)
                    .await?;
            } else {
                let name = if let Some(last) = created.last_name {
                    format!("{first} {last}", first = created.first_name)
                } else {
                    created.first_name
                };
                let text = format!("Добро пожаловать, <b>{name}</b>!");
                bot.send_message(msg.chat.id, text)
                    .parse_mode(teloxide::types::ParseMode::Html)
                    .await?;
            }
        }
    }
    Ok(())
}

async fn help(bot: Bot, msg: Message) -> Result<()> {
    bot.send_message(msg.chat.id, Command::descriptions().to_string())
        .await?;
    Ok(())
}
