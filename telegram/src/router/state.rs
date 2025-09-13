use anyhow::Result;
use client::Client;
use dptree::case;
use shared::models::Role;
use teloxide::{dispatching::DpHandlerDescription, prelude::*, types::InputFile};
use tracing::instrument;

use crate::{MyCallback, MyDialogue, State, TextCommand};

pub(super) fn router() -> Handler<'static, Result<()>, DpHandlerDescription> {
    Update::filter_message()
        .branch(case![State::TitleReceived].endpoint(title_received))
        .branch(case![State::ContentReceived { title }].endpoint(content_received))
        .branch(case![State::MediaReceived { title, content }].endpoint(media_received))
}
#[instrument(name = "title received", skip(bot, msg, dialogue, rpc_client))]
async fn title_received(
    bot: Bot,
    msg: Message,
    dialogue: MyDialogue,
    mut rpc_client: Client,
) -> Result<()> {
    if let Some(from) = msg.from.as_ref() {
        let id = from.id.0.try_into()?;
        let role = rpc_client
            .get_user(id)
            .await?
            .map(|u| u.role)
            .unwrap_or(Role::Guest);
        if role != Role::Guest {
            if let Some(message_text) = msg.text().as_ref() {
                let title = message_text.to_string();
                let text = format!("Заголовок: {title}\nПришлите содержание поста");
                tracing::info!("Title: {title}");
                bot.send_message(msg.chat.id, text)
                    .reply_markup(MyCallback::cancel_button())
                    .await?;
                dialogue
                    .update(State::ContentReceived { title: title })
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
#[instrument(name = "content received", skip(bot, msg, dialogue, rpc_client))]
async fn content_received(
    bot: Bot,
    msg: Message,
    dialogue: MyDialogue,
    mut rpc_client: Client,
) -> Result<()> {
    if let Some(from) = msg.from.as_ref() {
        let id = from.id.0.try_into()?;
        let role = rpc_client
            .get_user(id)
            .await?
            .map(|u| u.role)
            .unwrap_or(Role::Guest);
        if role != Role::Guest {
            if let Some(State::ContentReceived { title }) = dialogue.get().await? {
                if let Some(message_text) = msg.text().as_ref() {
                    let content = message_text.to_string();
                    tracing::info!("Title: {title}\nContent: {content}");
                    let text = format!(
                        "Заголовок: {title}\nСодержание:{content}\nПришлите медиа для поста или любое текст, чтобы сохранить без медиа"
                    );
                    bot.send_message(msg.chat.id, text)
                        .reply_markup(MyCallback::cancel_button())
                        .await?;
                    dialogue
                        .update(State::MediaReceived {
                            title: title,
                            content: content,
                        })
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
#[instrument(name = "media received", skip_all)]
async fn media_received(
    bot: Bot,
    msg: Message,
    dialogue: MyDialogue,
    mut rpc_client: Client,
) -> Result<()> {
    if let Some(from) = msg.from.as_ref() {
        let id = from.id.0.try_into()?;
        let role = rpc_client
            .get_user(id)
            .await?
            .map(|u| u.role)
            .unwrap_or(Role::Guest);
        if role != Role::Guest {
            if let Some(State::MediaReceived { title, content }) = dialogue.get().await? {
                tracing::info!("Title: {title}::Content:{content}");
                dialogue.exit().await?;
                let tg_photo_file_id = msg
                    .photo()
                    .and_then(|s| s.first())
                    .map(|f| f.file.id.0.clone());
                let tg_video_file_id = msg.video().map(|v| v.file.id.0.clone());

                let vk_photo_file_id = None;
                let vk_video_file_id = None;
                bot.delete_message(msg.chat.id, msg.id).await?;
                let post = rpc_client
                    .create_post(
                        id,
                        title,
                        content,
                        tg_photo_file_id,
                        vk_photo_file_id,
                        tg_video_file_id,
                        vk_video_file_id,
                    )
                    .await?;
                let text = format!(
                    "<b>{title}</b>\n{content}",
                    title = post.title,
                    content = post.content
                );
                let post_id = post.id;
                tracing::info!(
                    "Post ID: {post_id} size: {len}",
                    len = post_id.to_string().len()
                );
                let mu = MyCallback::not_published_kb(post_id);

                match post.tg_photo_file_id {
                    Some(file_id) => {
                        let photo = InputFile::file_id(file_id.into());
                        bot.send_photo(msg.chat.id, photo.clone())
                            .caption(text.clone())
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
                let markup = if role == Role::Admin {
                    TextCommand::admin_keyboard()
                } else {
                    TextCommand::editor_keyboard()
                };
                bot.send_message(msg.chat.id, "Чем еще могу помочь?")
                    .reply_markup(markup)
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
