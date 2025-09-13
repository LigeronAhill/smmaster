use std::str::FromStr;

use anyhow::{Result, anyhow};
use client::Client;
use dptree::case;
use shared::models::{Role, Status};
use teloxide::{dispatching::DpHandlerDescription, prelude::*, types::InputFile};

use crate::{MyCallback, MyDialogue, TextCommand};

pub(super) fn router() -> Handler<'static, Result<()>, DpHandlerDescription> {
    Update::filter_callback_query()
        .filter_map(|q: CallbackQuery| {
            let callback_str = q.data?;
            MyCallback::from_str(&callback_str).ok()
        })
        .branch(case![MyCallback::MakeUserEditor { id }].endpoint(make_editor))
        .branch(case![MyCallback::MakeUserGuest { id }].endpoint(make_guest))
        .branch(case![MyCallback::DeleteUser { id }].endpoint(delete_user))
        .branch(case![MyCallback::Cancel].endpoint(cancel))
        .branch(case![MyCallback::DeletePost { id }].endpoint(delete_post))
        .branch(
            case![MyCallback::PostsNextPage {
                author_id,
                status,
                page
            }]
            .endpoint(posts_page),
        )
}
async fn make_editor(
    bot: Bot,
    q: CallbackQuery,
    cb: MyCallback,
    mut rpc_client: Client,
) -> Result<()> {
    bot.answer_callback_query(q.id.clone()).await?;
    if let Some(msg) = q.regular_message() {
        let from = q.from.id.0.try_into()?;
        let role = rpc_client
            .get_user(from)
            .await?
            .map(|u| u.role)
            .unwrap_or(Role::Guest);
        if role != Role::Guest {
            if let MyCallback::MakeUserEditor { id } = cb {
                let mut existing = rpc_client
                    .get_user(id)
                    .await?
                    .ok_or(anyhow!("user not found"))?;
                existing.role = Role::Editor;
                let updated = rpc_client
                    .update_user(existing)
                    .await?
                    .ok_or(anyhow!("error making user editor"))?;
                let mu = MyCallback::editor_kb(updated.telegram_id);
                let name = if let Some(last) = updated.last_name {
                    format!("{first} {last}", first = updated.first_name)
                } else {
                    updated.first_name
                };

                let text = format!("{name}: {role}", role = updated.role);
                bot.edit_message_text(msg.chat.id, msg.id, text)
                    .reply_markup(mu)
                    .await?;
            }
        }
    }
    Ok(())
}
async fn make_guest(
    bot: Bot,
    q: CallbackQuery,
    cb: MyCallback,
    mut rpc_client: Client,
) -> Result<()> {
    bot.answer_callback_query(q.id.clone()).await?;
    if let Some(msg) = q.regular_message() {
        let from = q.from.id.0.try_into()?;
        let role = rpc_client
            .get_user(from)
            .await?
            .map(|u| u.role)
            .unwrap_or(Role::Guest);
        if role != Role::Guest {
            if let MyCallback::MakeUserGuest { id } = cb {
                let mut existing = rpc_client
                    .get_user(id)
                    .await?
                    .ok_or(anyhow!("user not found"))?;
                existing.role = Role::Guest;
                let updated = rpc_client
                    .update_user(existing)
                    .await?
                    .ok_or(anyhow!("error making user editor"))?;
                let mu = MyCallback::guest_kb(updated.telegram_id);
                let name = if let Some(last) = updated.last_name {
                    format!("{first} {last}", first = updated.first_name)
                } else {
                    updated.first_name
                };

                let text = format!("{name}: {role}", role = updated.role);
                bot.edit_message_text(msg.chat.id, msg.id, text)
                    .reply_markup(mu)
                    .await?;
            }
        }
    }
    Ok(())
}
async fn delete_user(
    bot: Bot,
    q: CallbackQuery,
    cb: MyCallback,
    mut rpc_client: Client,
) -> Result<()> {
    bot.answer_callback_query(q.id.clone()).await?;

    if let Some(msg) = q.regular_message() {
        let from = q.from.id.0.try_into()?;
        let role = rpc_client
            .get_user(from)
            .await?
            .map(|u| u.role)
            .unwrap_or(Role::Guest);
        if role != Role::Guest {
            if let MyCallback::DeleteUser { id } = cb {
                let existing = rpc_client
                    .get_user(id)
                    .await?
                    .ok_or(anyhow!("user not found"))?;
                let result = rpc_client.delete_user(existing.telegram_id).await?;
                if result {
                    bot.delete_message(msg.chat.id, msg.id).await?;
                }
            }
        }
    }
    Ok(())
}
async fn cancel(
    bot: Bot,
    q: CallbackQuery,
    dialogue: MyDialogue,
    mut rpc_client: Client,
) -> Result<()> {
    bot.answer_callback_query(q.id.clone()).await?;
    dialogue.exit().await?;
    if let Some(msg) = q.regular_message() {
        let from = q.from.id.0.try_into()?;
        let role = rpc_client
            .get_user(from)
            .await?
            .map(|u| u.role)
            .unwrap_or(Role::Guest);
        let mu = match role {
            Role::Guest => TextCommand::guest_keyboard(),
            Role::Editor => TextCommand::editor_keyboard(),
            Role::Admin => TextCommand::admin_keyboard(),
        };
        bot.send_message(msg.chat.id, "Действие отменено")
            .reply_markup(mu)
            .await?;
    }
    Ok(())
}
async fn delete_post(
    bot: Bot,
    q: CallbackQuery,
    cb: MyCallback,
    mut rpc_client: Client,
) -> Result<()> {
    bot.answer_callback_query(q.id.clone()).await?;
    if let Some(msg) = q.regular_message() {
        let from = q.from.id.0.try_into()?;
        let role = rpc_client
            .get_user(from)
            .await?
            .map(|u| u.role)
            .unwrap_or(Role::Guest);
        if role != Role::Guest {
            if let MyCallback::DeletePost { id } = cb {
                rpc_client.delete_post(id).await?;
                bot.delete_message(msg.chat.id, msg.id).await?;
            }
        }
    }
    Ok(())
}
async fn posts_page(
    bot: Bot,
    q: CallbackQuery,
    cb: MyCallback,
    mut rpc_client: Client,
) -> Result<()> {
    bot.answer_callback_query(q.id.clone()).await?;
    if let Some(msg) = q.regular_message() {
        let from = q.from.id.0.try_into()?;
        let role = rpc_client
            .get_user(from)
            .await?
            .map(|u| u.role)
            .unwrap_or(Role::Guest);
        if role != Role::Guest {
            match cb {
                MyCallback::PostsNextPage {
                    author_id,
                    status,
                    page,
                }
                | MyCallback::PostsPreviousPage {
                    author_id,
                    status,
                    page,
                } => {
                    let (posts, has_next) = match status {
                        shared::models::Status::Draft => rpc_client.drafts(author_id, page).await?,
                        shared::models::Status::Pending => {
                            rpc_client.pending(author_id, page).await?
                        }
                        shared::models::Status::Published => {
                            rpc_client.published(author_id, page).await?
                        }
                        shared::models::Status::Abandoned => (Vec::new(), false),
                    };
                    for post in posts {
                        let text = match status {
                            shared::models::Status::Pending => {
                                format!(
                                    "<b>{title}</b>\n{content}\nОпубликую: {date}",
                                    title = post.title,
                                    content = post.content,
                                    date = post.publish_datetime.unwrap_or_default().to_rfc3339(),
                                )
                            }
                            shared::models::Status::Published => {
                                format!(
                                    "<b>{title}</b>\n{content}\nОпубликован: {date}",
                                    title = post.title,
                                    content = post.content,
                                    date = post.publish_datetime.unwrap_or_default().to_rfc3339(),
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
                        let post_id = post.id;
                        let mu = if status != Status::Published {
                            MyCallback::not_published_kb(post_id)
                        } else {
                            MyCallback::published_kb(post_id)
                        };
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
                    if page != 1 {
                        if has_next {
                            bot.send_message(msg.chat.id, "Это не все")
                                .reply_markup(MyCallback::has_previous_and_next_kb(
                                    author_id, status, page,
                                ))
                                .await?;
                        } else {
                            bot.send_message(msg.chat.id, "Это все")
                                .reply_markup(MyCallback::has_previous_kb(
                                    author_id,
                                    status,
                                    page - 1,
                                ))
                                .await?;
                        }
                    } else {
                        if has_next {
                            bot.send_message(msg.chat.id, "Это не все")
                                .reply_markup(MyCallback::has_next_kb(author_id, status, page + 1))
                                .await?;
                        } else {
                            bot.send_message(msg.chat.id, "Это все")
                                .reply_markup(MyCallback::cancel_button())
                                .await?;
                        }
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
}
