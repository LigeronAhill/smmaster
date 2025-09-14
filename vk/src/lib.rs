use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use shared::models::Post;
use tracing::instrument;

const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
const BASE_URL: &str = "https://api.vk.ru/method";

#[derive(Clone)]
pub struct VKClient {
    client: reqwest::Client,
    token: String,
    group_id: i64,
    version: String,
    photo_album: i64,
}
impl VKClient {
    #[instrument(name = "new vk client", skip(token))]
    pub async fn new(token: String, group_id: i64) -> Result<Self> {
        let client = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .gzip(true)
            .build()?;
        let version = "5.199".to_string();
        let url = format!("{BASE_URL}/photos.getAlbums?owner_id=-{group_id}&v={version}");
        let response: serde_json::Value = client
            .get(url)
            .bearer_auth(&token)
            .send()
            .await?
            .json()
            .await?;

        match serde_json::from_value::<ApiResponse<AlbumsResponse>>(response.clone()) {
            Ok(albums) => {
                let photo_album = albums
                    .response
                    .items
                    .first()
                    .map(|a| a.id)
                    .ok_or(anyhow!("no photo album!"))?;
                Ok(Self {
                    client,
                    token,
                    group_id,
                    version,
                    photo_album,
                })
            }
            Err(e) => {
                let err = format!("Error: {e:?}\nResponse:\n{response:#?}");
                Err(anyhow!(err))
            }
        }
    }
    #[instrument(name = "publish post", skip(self))]
    pub async fn publish(&self, post: &Post) -> Result<()> {
        let response = if let Some(photo) = post.vk_photo_file_id.as_ref() {
            // Has photo
            let url = format!(
                "{BASE_URL}/wall.post?owner_id=-{gid}&message={msg}&v={v}&attachments={photo}",
                gid = self.group_id,
                msg = post.content,
                v = self.version,
            );
            self.client.get(url).bearer_auth(&self.token).send().await?
        } else if let Some(video) = post.vk_video_file_id.as_ref() {
            // Has video
            let url = format!(
                "{BASE_URL}/wall.post?owner_id=-{gid}&message={msg}&v={v}&attachments={video}",
                gid = self.group_id,
                msg = post.content,
                v = self.version,
            );
            self.client.get(url).bearer_auth(&self.token).send().await?
        } else {
            let url = format!(
                "{BASE_URL}/wall.post?owner_id=-{gid}&message={msg}&v={v}",
                gid = self.group_id,
                msg = post.content,
                v = self.version
            );
            self.client.get(url).bearer_auth(&self.token).send().await?
        };
        if response.status().is_success() {
            Ok(())
        } else {
            let err = response.text().await?;
            Err(anyhow!(err))
        }
    }
    #[instrument(name = "get vk photo id", skip(self))]
    pub async fn get_photo_id(&self, file_path: String) -> Result<String> {
        let upload_url = self.get_photo_upload_url().await?;
        let ufr = self.upload_photo(upload_url, file_path).await?;
        let photo_id = self.save_photo(ufr).await?;
        Ok(photo_id)
    }
    #[instrument(name = "get vk video id", skip(self))]
    pub async fn get_video_id(&self, file_path: String) -> Result<String> {
        let upload_url = self.get_video_upload_url().await?;
        let photo_id = self.upload_video(upload_url, file_path).await?;
        Ok(photo_id)
    }
    #[instrument(name = "get vk photo upload url", skip(self))]
    async fn get_photo_upload_url(&self) -> Result<String> {
        let url = format!(
            "{BASE_URL}/photos.getUploadServer?group_id={gid}&v={v}&album_id={aid}",
            gid = self.group_id,
            v = self.version,
            aid = self.photo_album
        );
        let response: serde_json::Value = self
            .client
            .get(url)
            .bearer_auth(&self.token)
            .send()
            .await?
            .json()
            .await?;
        match serde_json::from_value::<ApiResponse<PhotoUploadUrlResponse>>(response.clone()) {
            Ok(url_res) => Ok(url_res.response.upload_url),
            Err(e) => {
                let err = format!("Error: {e:?}\nResponse:\n{response:#?}");
                Err(anyhow!(err))
            }
        }
    }
    #[instrument(name = "upload photo to vk", skip(self))]
    async fn upload_photo(
        &self,
        upload_url: String,
        file_path: String,
    ) -> Result<UploadFileResponse> {
        let form = reqwest::multipart::Form::new()
            .file("file1", file_path)
            .await?;
        let response: serde_json::Value = self
            .client
            .post(upload_url)
            .bearer_auth(&self.token)
            .multipart(form)
            .send()
            .await?
            .json()
            .await?;
        match serde_json::from_value::<UploadFileResponse>(response.clone()) {
            Ok(mut res) => {
                res.photos_list = res.photos_list.replace('\\', "");
                Ok(res)
            }
            Err(e) => {
                let err = format!("Error: {e:?}\nResponse:\n{response:#?}");
                Err(anyhow!(err))
            }
        }
    }
    #[instrument(name = "save photo to vk", skip(self, ufr))]
    async fn save_photo(&self, ufr: UploadFileResponse) -> Result<String> {
        let url = format!("{BASE_URL}/photos.save");
        let form = reqwest::multipart::Form::new()
            .text("server", ufr.server.to_string())
            .text("photos_list", ufr.photos_list)
            .text("hash", ufr.hash)
            .text("v", self.version.clone())
            .text("album_id", self.photo_album.to_string())
            .text("group_id", self.group_id.to_string())
            .text("access_token", self.token.clone());
        let response: serde_json::Value = self
            .client
            .post(url)
            .multipart(form)
            .bearer_auth(&self.token)
            .send()
            .await?
            .json()
            .await?;
        match serde_json::from_value::<ApiResponse<Vec<SavePhotoResponse>>>(response.clone()) {
            Ok(response) => {
                let id = response
                    .response
                    .last()
                    .ok_or(anyhow!("no photos saved"))?
                    .id;
                let res = format!("photo-{gid}_{id}", gid = self.group_id);
                Ok(res)
            }
            Err(e) => {
                let err = format!("Error: {e:?}\nResponse:\n{response:#?}");
                Err(anyhow!(err))
            }
        }
    }
    #[instrument(name = "get vk video upload url", skip(self))]
    async fn get_video_upload_url(&self) -> Result<String> {
        let url = format!(
            "{BASE_URL}/video.save?group_id={gid}&v={v}",
            gid = self.group_id,
            v = self.version,
        );
        let response: serde_json::Value = self
            .client
            .get(url)
            .bearer_auth(&self.token)
            .send()
            .await?
            .json()
            .await?;
        match serde_json::from_value::<ApiResponse<VideoUploadUrlResponse>>(response.clone()) {
            Ok(url_res) => Ok(url_res.response.upload_url),
            Err(e) => {
                let err = format!("Error: {e:?}\nResponse:\n{response:#?}");
                Err(anyhow!(err))
            }
        }
    }
    #[instrument(name = "upload photo to vk", skip(self))]
    async fn upload_video(&self, upload_url: String, file_path: String) -> Result<String> {
        let form = reqwest::multipart::Form::new()
            .file("file1", file_path)
            .await?;
        let response: serde_json::Value = self
            .client
            .post(upload_url)
            .bearer_auth(&self.token)
            .multipart(form)
            .send()
            .await?
            .json()
            .await?;
        match serde_json::from_value::<UploadedVideoResponse>(response.clone()) {
            Ok(res) => {
                let id = res.video_id;
                let res = format!("video-{gid}_{id}", gid = self.group_id);
                Ok(res)
            }
            Err(e) => {
                let err = format!("Error: {e:?}\nResponse:\n{response:#?}");
                Err(anyhow!(err))
            }
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub response: T,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlbumsResponse {
    pub count: i64,
    pub items: Vec<Album>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Album {
    pub id: i64,
    pub owner_id: i64,
    pub size: i64,
    pub title: String,
    pub feed_disabled: i64,
    pub feed_has_pinned: i64,
    pub can_upload: i64,
    pub comments_disabled: i64,
    pub created: i64,
    pub description: String,
    pub can_delete: bool,
    pub can_include_to_feed: bool,
    pub thumb_id: i64,
    pub thumb_is_last: i64,
    pub updated: i64,
    pub upload_by_admins_only: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhotoUploadUrlResponse {
    pub album_id: i64,
    pub upload_url: String,
    pub user_id: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UploadFileResponse {
    pub server: i64,
    pub photos_list: String,
    pub aid: i64,
    pub hash: String,
    pub gid: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SavePhotoResponse {
    pub album_id: i64,
    pub date: i64,
    pub id: i64,
    pub owner_id: i64,
    pub sizes: Vec<Size>,
    pub text: String,
    pub user_id: i64,
    pub web_view_token: String,
    pub has_tags: bool,
    pub orig_photo: OrigPhoto,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Size {
    pub height: i64,
    #[serde(rename = "type")]
    pub type_field: String,
    pub width: i64,
    pub url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrigPhoto {
    pub height: i64,
    #[serde(rename = "type")]
    pub type_field: String,
    pub url: String,
    pub width: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VideoUploadUrlResponse {
    pub access_key: String,
    pub access_by_link_key: String,
    pub description: String,
    pub owner_id: i64,
    pub title: String,
    pub generated_title: String,
    pub upload_url: String,
    pub video_id: i64,
}
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UploadedVideoResponse {
    pub video_hash: String,
    pub size: i64,
    pub direct_link: String,
    pub owner_id: i64,
    pub video_id: i64,
}
