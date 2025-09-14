use std::str::FromStr;

use anyhow::{Result, anyhow};
use client::Client;
use dptree::case;
use shared::models::{Role, Status};
use teloxide::{
    dispatching::DpHandlerDescription, prelude::*, sugar::bot::BotMessagesExt,
    types::KeyboardRemove,
};

use crate::{MyCallback, MyDialogue, TextCommand, moscow, send_post};

pub(super) fn router() -> Handler<'static, Result<()>, DpHandlerDescription> {
    Update::filter_callback_query()
        .filter_map(|q: CallbackQuery| {
            let callback_str = q.data?;
            MyCallback::from_str(&callback_str).ok()
        })
        // Users
        .branch(case![MyCallback::MakeUserEditor { id }].endpoint(make_editor))
        .branch(case![MyCallback::MakeUserGuest { id }].endpoint(make_guest))
        .branch(case![MyCallback::DeleteUser { id }].endpoint(delete_user))
        .branch(case![MyCallback::Drafts { author_id }].endpoint(users_drafts))
        .branch(case![MyCallback::Pending { author_id }].endpoint(users_pending))
        .branch(case![MyCallback::Published { author_id }].endpoint(users_published))
        // Posts
        .branch(case![MyCallback::PublishNow { id }].endpoint(publish_post))
        .branch(case![MyCallback::SetPublishDate { id }].endpoint(set_publish_date))
        .branch(case![MyCallback::DeletePost { id }].endpoint(delete_post))
        .branch(
            case![MyCallback::PostsNextPage {
                author_id,
                status,
                page
            }]
            .endpoint(posts_page),
        )
        .branch(
            case![MyCallback::PostsPreviousPage {
                author_id,
                status,
                page
            }]
            .endpoint(posts_page),
        )
        // Cancel
        .branch(case![MyCallback::Cancel].endpoint(cancel))
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
        } else {
            bot.send_message(msg.chat.id, "У вас нет доступа")
                .reply_markup(TextCommand::guest_keyboard())
                .await?;
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
        } else {
            bot.send_message(msg.chat.id, "У вас нет доступа")
                .reply_markup(TextCommand::guest_keyboard())
                .await?;
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
        } else {
            bot.send_message(msg.chat.id, "У вас нет доступа")
                .reply_markup(TextCommand::guest_keyboard())
                .await?;
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
                let mu = if role == Role::Admin {
                    TextCommand::admin_keyboard()
                } else {
                    TextCommand::editor_keyboard()
                };
                bot.send_message(msg.chat.id, "Пост удален")
                    .reply_markup(mu)
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
                        send_post(&bot, msg, &post).await?;
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
        } else {
            bot.send_message(msg.chat.id, "У вас нет доступа")
                .reply_markup(TextCommand::guest_keyboard())
                .await?;
        }
    }
    Ok(())
}
async fn users_drafts(
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
        if role == Role::Admin {
            if let MyCallback::Drafts { author_id } = cb {
                let (posts, has_next) = rpc_client.drafts(author_id, 1).await?;
                for post in posts {
                    send_post(&bot, msg, &post).await?;
                }
                if has_next {
                    bot.send_message(msg.chat.id, "Это не все")
                        .reply_markup(MyCallback::has_next_kb(author_id, Status::Draft, 2))
                        .await?;
                } else {
                    bot.send_message(msg.chat.id, "Это все")
                        .reply_markup(MyCallback::cancel_button())
                        .await?;
                }
            }
        } else {
            bot.send_message(msg.chat.id, "У вас нет доступа")
                .reply_markup(TextCommand::guest_keyboard())
                .await?;
        }
    }
    Ok(())
}
async fn users_pending(
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
        if role == Role::Admin {
            if let MyCallback::Pending { author_id } = cb {
                let (posts, has_next) = rpc_client.pending(author_id, 1).await?;
                for post in posts {
                    send_post(&bot, msg, &post).await?;
                }
                if has_next {
                    bot.send_message(msg.chat.id, "Это не все")
                        .reply_markup(MyCallback::has_next_kb(author_id, Status::Pending, 2))
                        .await?;
                } else {
                    bot.send_message(msg.chat.id, "Это все")
                        .reply_markup(MyCallback::cancel_button())
                        .await?;
                }
            }
        } else {
            bot.send_message(msg.chat.id, "У вас нет доступа")
                .reply_markup(TextCommand::guest_keyboard())
                .await?;
        }
    }
    Ok(())
}
async fn users_published(
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
        if role == Role::Admin {
            if let MyCallback::Published { author_id } = cb {
                let (posts, has_next) = rpc_client.published(author_id, 1).await?;
                for post in posts {
                    send_post(&bot, msg, &post).await?;
                }
                if has_next {
                    bot.send_message(msg.chat.id, "Это не все")
                        .reply_markup(MyCallback::has_next_kb(author_id, Status::Published, 2))
                        .await?;
                } else {
                    bot.send_message(msg.chat.id, "Это все")
                        .reply_markup(MyCallback::cancel_button())
                        .await?;
                }
            }
        } else {
            bot.send_message(msg.chat.id, "У вас нет доступа")
                .reply_markup(TextCommand::guest_keyboard())
                .await?;
        }
    }
    Ok(())
}
async fn publish_post(
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
            if let MyCallback::PublishNow { id } = cb {
                let now = chrono::Utc::now();
                let post = rpc_client
                    .set_publish_date(id, now)
                    .await?
                    .ok_or(anyhow!("Error publishing post"))?;
                let text = format!(
                    "<b>{title}</b>\n{content}\nОпубликован: {date}",
                    title = post.title,
                    content = post.content,
                    date = moscow(post.publish_datetime.unwrap_or_default()),
                );
                let mu = MyCallback::published_kb(post.id);
                if bot
                    .edit_message_text(msg.chat.id, msg.id, &text)
                    .reply_markup(mu.clone())
                    .parse_mode(teloxide::types::ParseMode::Html)
                    .await
                    .is_err()
                {
                    bot.edit_caption(msg)
                        .caption(text)
                        .reply_markup(mu)
                        .parse_mode(teloxide::types::ParseMode::Html)
                        .await?;
                }
            }
        } else {
            bot.send_message(msg.chat.id, "У вас нет доступа")
                .reply_markup(TextCommand::guest_keyboard())
                .await?;
        }
    }
    Ok(())
}
async fn set_publish_date(
    bot: Bot,
    q: CallbackQuery,
    dialogue: MyDialogue,
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
            if let MyCallback::SetPublishDate { id } = cb {
                let text = "Пришлите дату и время в формате '2025-09-14 10:15'";
                bot.send_message(msg.chat.id, text)
                    .reply_markup(KeyboardRemove::new())
                    .await?;
                dialogue
                    .update(crate::State::PublishDateReceive { post_id: id })
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
