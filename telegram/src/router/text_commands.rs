use std::str::FromStr;

use anyhow::Result;
use client::Client;
use dptree::case;
use shared::models::{Role, Status};
use teloxide::{
    dispatching::DpHandlerDescription,
    prelude::*,
    types::{InputFile, KeyboardRemove},
};

use crate::{MyCallback, MyDialogue, TextCommand};

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
            dialogue.update(crate::State::TitleReceived).await?;
        } else {
            bot.send_message(msg.chat.id, "У вас нет доступа")
                .reply_markup(TextCommand::guest_keyboard())
                .await?;
        }
    }
    Ok(())
}
async fn drafts(bot: Bot, msg: Message, mut rpc_client: Client) -> Result<()> {
    if let Some(from) = msg.from {
        let id = from.id.0.try_into()?;
        let role = rpc_client
            .get_user(id)
            .await?
            .map(|u| u.role)
            .unwrap_or(Role::Guest);
        if role != Role::Guest {
            let (posts, has_next) = rpc_client.drafts(id, 1).await?;
            for post in posts {
                let text = format!(
                    "<b>{title}</b>\n{content}",
                    title = post.title,
                    content = post.content
                );
                let post_id = post.id;
                let mu = MyCallback::not_published_kb(post_id);
                match post.tg_photo_file_id {
                    Some(file_id) => {
                        let photo = InputFile::file_id(file_id.into());
                        bot.send_photo(msg.chat.id, photo)
                            .caption(text)
                            .reply_markup(mu)
                            .parse_mode(teloxide::types::ParseMode::Html)
                            .await?;
                    }
                    None => {
                        // TODO: video?
                        bot.send_message(msg.chat.id, text)
                            .reply_markup(mu)
                            .parse_mode(teloxide::types::ParseMode::Html)
                            .await?;
                    }
                }
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
    if let Some(from) = msg.from {
        let id = from.id.0.try_into()?;
        let role = rpc_client
            .get_user(id)
            .await?
            .map(|u| u.role)
            .unwrap_or(Role::Guest);
        if role != Role::Guest {
            let (posts, has_next) = rpc_client.pending(id, 1).await?;
            for post in posts {
                let text = format!(
                    "<b>{title}</b>\n{content}\nОпубликую: {date}",
                    title = post.title,
                    content = post.content,
                    date = post.publish_datetime.unwrap_or_default().to_rfc3339(),
                );
                let post_id = post.id;
                let mu = MyCallback::not_published_kb(post_id);
                match post.tg_photo_file_id {
                    Some(file_id) => {
                        let photo = InputFile::file_id(file_id.into());
                        bot.send_photo(msg.chat.id, photo)
                            .caption(text)
                            .parse_mode(teloxide::types::ParseMode::Html)
                            .reply_markup(mu)
                            .await?;
                    }
                    None => {
                        // TODO: video?
                        bot.send_message(msg.chat.id, text)
                            .reply_markup(mu)
                            .parse_mode(teloxide::types::ParseMode::Html)
                            .await?;
                    }
                }
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
    if let Some(from) = msg.from {
        let id = from.id.0.try_into()?;
        let role = rpc_client
            .get_user(id)
            .await?
            .map(|u| u.role)
            .unwrap_or(Role::Guest);
        if role != Role::Guest {
            let (posts, has_next) = rpc_client.published(id, 1).await?;
            for post in posts {
                let text = format!(
                    "<b>{title}</b>\n{content}\nОпубликован: {date}",
                    title = post.title,
                    content = post.content,
                    date = post.publish_datetime.unwrap_or_default().to_rfc3339(),
                );
                let post_id = post.id;
                let mu = MyCallback::published_kb(post_id);
                match post.tg_photo_file_id {
                    Some(file_id) => {
                        let photo = InputFile::file_id(file_id.into());
                        bot.send_photo(msg.chat.id, photo)
                            .caption(text)
                            .reply_markup(mu)
                            .parse_mode(teloxide::types::ParseMode::Html)
                            .await?;
                    }
                    None => {
                        // TODO: video?
                        bot.send_message(msg.chat.id, text)
                            .reply_markup(mu)
                            .parse_mode(teloxide::types::ParseMode::Html)
                            .await?;
                    }
                }
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
