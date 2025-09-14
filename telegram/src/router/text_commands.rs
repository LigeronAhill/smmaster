use std::str::FromStr;

use anyhow::Result;
use client::Client;
use dptree::case;
use shared::models::{Role, Status};
use teloxide::{dispatching::DpHandlerDescription, prelude::*, types::KeyboardRemove};

use crate::{MyCallback, MyDialogue, TextCommand, send_post};

pub(super) fn router() -> Handler<'static, Result<()>, DpHandlerDescription> {
    Update::filter_message()
        .filter_map(|msg: Message| {
            let text_command_str = msg.text()?;
            TextCommand::from_str(text_command_str).ok()
        })
        .branch(case![TextCommand::Users].endpoint(users))
        .branch(case![TextCommand::CreatePost].endpoint(new_post))
        .branch(case![TextCommand::Drafts].endpoint(drafts))
        .branch(case![TextCommand::Pending].endpoint(pending))
        .branch(case![TextCommand::Published].endpoint(published))
}

async fn users(bot: Bot, msg: Message, mut rpc_client: Client) -> Result<()> {
    if let Some(user_id) = msg.from.and_then(|f| f.id.0.try_into().ok()) {
        let role = rpc_client
            .get_user(user_id)
            .await?
            .map(|u| u.role)
            .unwrap_or(Role::Guest);
        if role == Role::Admin {
            let (users, _has_next) = rpc_client.list_users(1).await?;
            for user in users {
                if user.role == Role::Admin {
                    continue;
                }
                let name = if let Some(last) = user.last_name {
                    format!("{first} {last}", first = user.first_name)
                } else {
                    user.first_name
                };
                let mu = if user.role == Role::Guest {
                    MyCallback::guest_kb(user.telegram_id)
                } else {
                    MyCallback::editor_kb(user.telegram_id)
                };
                bot.send_message(msg.chat.id, format!("{name}: {role}", role = user.role))
                    .reply_markup(mu)
                    .await?;
            }
        }
    }
    Ok(())
}
async fn new_post(
    bot: Bot,
    msg: Message,
    mut rpc_client: Client,
    dialogue: MyDialogue,
) -> Result<()> {
    if let Some(from) = msg.from {
        let id = from.id.0.try_into()?;
        let role = rpc_client
            .get_user(id)
            .await?
            .map(|u| u.role)
            .unwrap_or(Role::Guest);
        if role != Role::Guest {
            bot.send_message(msg.chat.id, "Начнем")
                .reply_markup(KeyboardRemove::new())
                .await?;
            bot.send_message(msg.chat.id, "Пришлите название поста")
                .reply_markup(MyCallback::cancel_button())
                .await?;
            dialogue.update(crate::State::TitleReceive).await?;
        } else {
            bot.send_message(msg.chat.id, "У вас нет доступа")
                .reply_markup(TextCommand::guest_keyboard())
                .await?;
        }
    }
    Ok(())
}
async fn drafts(bot: Bot, msg: Message, mut rpc_client: Client) -> Result<()> {
    if let Some(from) = msg.from.as_ref() {
        let id = from.id.0.try_into()?;
        let role = rpc_client
            .get_user(id)
            .await?
            .map(|u| u.role)
            .unwrap_or(Role::Guest);
        if role != Role::Guest {
            let (posts, has_next) = rpc_client.drafts(id, 1).await?;
            for post in posts {
                send_post(&bot, &msg, &post).await?;
            }
            if has_next {
                bot.send_message(msg.chat.id, "Это не все")
                    .reply_markup(MyCallback::has_next_kb(id, Status::Draft, 2))
                    .await?;
            } else {
                bot.send_message(msg.chat.id, "Это все")
                    .reply_markup(MyCallback::cancel_button())
                    .await?;
            }
        } else {
            bot.send_message(msg.chat.id, "У вас нет доступа")
                .reply_markup(TextCommand::guest_keyboard())
                .await?;
        }
    }

    Ok(())
}
async fn pending(bot: Bot, msg: Message, mut rpc_client: Client) -> Result<()> {
    if let Some(from) = msg.from.as_ref() {
        let id = from.id.0.try_into()?;
        let role = rpc_client
            .get_user(id)
            .await?
            .map(|u| u.role)
            .unwrap_or(Role::Guest);
        if role != Role::Guest {
            let (posts, has_next) = rpc_client.pending(id, 1).await?;
            for post in posts {
                send_post(&bot, &msg, &post).await?;
            }
            if has_next {
                bot.send_message(msg.chat.id, "Это не все")
                    .reply_markup(MyCallback::has_next_kb(id, Status::Pending, 2))
                    .await?;
            } else {
                bot.send_message(msg.chat.id, "Это все")
                    .reply_markup(MyCallback::cancel_button())
                    .await?;
            }
        } else {
            bot.send_message(msg.chat.id, "У вас нет доступа")
                .reply_markup(TextCommand::guest_keyboard())
                .await?;
        }
    }

    Ok(())
}
async fn published(bot: Bot, msg: Message, mut rpc_client: Client) -> Result<()> {
    if let Some(from) = msg.from.as_ref() {
        let id = from.id.0.try_into()?;
        let role = rpc_client
            .get_user(id)
            .await?
            .map(|u| u.role)
            .unwrap_or(Role::Guest);
        if role != Role::Guest {
            let (posts, has_next) = rpc_client.published(id, 1).await?;
            for post in posts {
                send_post(&bot, &msg, &post).await?;
            }
            if has_next {
                bot.send_message(msg.chat.id, "Это не все")
                    .reply_markup(MyCallback::has_next_kb(id, Status::Published, 2))
                    .await?;
            } else {
                bot.send_message(msg.chat.id, "Это все")
                    .reply_markup(MyCallback::cancel_button())
                    .await?;
            }
        } else {
            bot.send_message(msg.chat.id, "У вас нет доступа")
                .reply_markup(TextCommand::guest_keyboard())
                .await?;
        }
    }

    Ok(())
}
