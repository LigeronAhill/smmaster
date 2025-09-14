use anyhow::Result;
use shared::models::Post;
use teloxide::{
    prelude::*,
    types::{ChatId, FileId, InputFile},
};

#[derive(Clone)]
pub struct Publisher {
    tg: teloxide::Bot,
    tg_channel: ChatId,
    rpc_client: client::Client,
    vk_client: vk::VKClient,
}
impl Publisher {
    pub fn new(
        bot: teloxide::Bot,
        tg_channel_id: i64,
        rpc_client: client::Client,
        vk_client: vk::VKClient,
    ) -> Self {
        let tg_channel = ChatId(tg_channel_id);
        Self {
            tg: bot,
            tg_channel,
            rpc_client,
            vk_client,
        }
    }
    pub async fn run(self) {
        let mut p = self.clone();
        loop {
            if let Err(e) = p.process().await {
                tracing::error!("Error running publisher: {e:?}");
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        }
    }
    async fn publish(&self, post: Post) -> Result<()> {
        let mut client = self.rpc_client.clone();
        if let Some(post) = client.publish_now(post.id).await? {
            // VK
            self.vk_client.publish(&post).await?;

            // Telegram
            if let Some(photo) = post.tg_photo_file_id {
                let photo = InputFile::file_id(FileId::from(photo));
                self.tg
                    .send_photo(self.tg_channel, photo)
                    .caption(post.content)
                    .await?;
            } else if let Some(video) = post.tg_video_file_id {
                let video = InputFile::file_id(FileId::from(video));
                self.tg
                    .send_video(self.tg_channel, video)
                    .caption(post.content)
                    .await?;
            } else {
                self.tg.send_message(self.tg_channel, post.content).await?;
            }
        }
        Ok(())
    }
    async fn process(&mut self) -> Result<()> {
        let mut authors = Vec::new();
        let mut page = 1;
        loop {
            let (current_authors, has_next) = self.rpc_client.list_users(page).await?;
            authors.extend(current_authors);
            if has_next {
                page += 1;
            } else {
                break;
            }
        }
        for author in authors {
            let mut page = 1;
            let mut pendings = Vec::new();
            loop {
                let (current_pendings, has_next) =
                    self.rpc_client.pending(author.telegram_id, page).await?;
                pendings.extend(current_pendings);
                if has_next {
                    page += 1;
                } else {
                    break;
                }
            }
            for pending in pendings {
                if let Some(pd) = pending.publish_datetime {
                    let now = chrono::Utc::now();
                    if pd <= now {
                        self.publish(pending).await?;
                    }
                }
            }
        }
        Ok(())
    }
}
