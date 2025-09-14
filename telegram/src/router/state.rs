use anyhow::{Result, anyhow};
use client::Client;
use dptree::case;
use shared::models::Role;
use teloxide::{dispatching::DpHandlerDescription, net::Download, prelude::*};
use tracing::instrument;

use crate::{MyCallback, MyDialogue, State, TextCommand, send_post, to_utc};

pub(super) fn router() -> Handler<'static, Result<()>, DpHandlerDescription> {
    Update::filter_message()
        .branch(case![State::TitleReceive].endpoint(title_received))
        .branch(case![State::ContentReceive { title }].endpoint(content_received))
        .branch(case![State::MediaReceive { title, content }].endpoint(media_received))
        .branch(case![State::PublishDateReceive { post_id }].endpoint(publish_date_received))
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
                // TODO: CHECK TITLE!!!
                let text = format!("Заголовок: {title}\nПришлите содержание поста");
                tracing::info!("Title: {title}");
                bot.send_message(msg.chat.id, text)
                    .reply_markup(MyCallback::cancel_button())
                    .await?;
                dialogue
                    .update(State::ContentReceive { title: title })
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
            if let Some(State::ContentReceive { title }) = dialogue.get().await? {
                if let Some(message_text) = msg.text().as_ref() {
                    let content = message_text.to_string();
                    // TODO: CHECK CONTENT!!!
                    let text = format!(
                        "Заголовок: {title}\nСодержание:{content}\nПришлите медиа для поста или любое текст, чтобы сохранить без медиа"
                    );
                    bot.send_message(msg.chat.id, text)
                        .reply_markup(MyCallback::cancel_button())
                        .await?;
                    dialogue
                        .update(State::MediaReceive {
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
    vk_client: vk::VKClient,
) -> Result<()> {
    if let Some(from) = msg.from.as_ref() {
        let id = from.id.0.try_into()?;
        let role = rpc_client
            .get_user(id)
            .await?
            .map(|u| u.role)
            .unwrap_or(Role::Guest);
        if role != Role::Guest {
            if let Some(State::MediaReceive { title, content }) = dialogue.get().await? {
                tracing::info!("Title: {title} :: Content:{content}");
                dialogue.exit().await?;
                let tg_photo_file_id = msg
                    .photo()
                    .and_then(|s| s.first())
                    .map(|f| f.file.id.0.clone());
                let mut vk_photo_file_id = None;
                if let Some(ps) = msg.photo().and_then(|p| p.last()) {
                    let file = bot.get_file(ps.file.id.clone()).await?;
                    let extension = file.path.split('.').last().unwrap_or_default();
                    let path = format!("/tmp/photo.{extension}");
                    let mut dst = tokio::fs::File::create(&path).await?;
                    bot.download_file(&file.path, &mut dst).await?;
                    match vk_client.get_photo_id(path).await {
                        Ok(id) => vk_photo_file_id = Some(id),
                        Err(e) => tracing::error!("{e:?}"),
                    }
                }
                let tg_video_file_id = msg.video().map(|v| v.file.id.0.clone());
                let mut vk_video_file_id = None;
                if let Some(vs) = msg.video() {
                    let file = bot.get_file(vs.file.id.clone()).await?;
                    let extension = file.path.split('.').last().unwrap_or_default();
                    let path = format!("/tmp/video.{extension}");
                    let mut dst = tokio::fs::File::create(&path).await?;
                    bot.download_file(&file.path, &mut dst).await?;
                    match vk_client.get_video_id(path).await {
                        Ok(id) => vk_video_file_id = Some(id),
                        Err(e) => tracing::error!("{e:?}"),
                    }
                }

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
                send_post(&bot, &msg, &post).await?;
                let mu = if role == Role::Admin {
                    TextCommand::admin_keyboard()
                } else {
                    TextCommand::editor_keyboard()
                };
                bot.send_message(msg.chat.id, "Могу я еще чем-то помочь?")
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
#[instrument(name = "publish date received", skip(bot, msg, dialogue, rpc_client))]
async fn publish_date_received(
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
                if let Some(State::PublishDateReceive { post_id }) = dialogue.get().await? {
                    let date = to_utc(&message_text)?;
                    let post = rpc_client
                        .set_publish_date(post_id, date)
                        .await?
                        .ok_or(anyhow!("Error setting post publish date"))?;
                    send_post(&bot, &msg, &post).await?;
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
