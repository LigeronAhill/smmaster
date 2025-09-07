use std::fmt::Display;

use anyhow::anyhow;
use bson::serde_helpers::{datetime, uuid_1};
use chrono::{DateTime, Utc};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
#[builder(build_fn(validate = "Self::validate"))]
// Сообщение, представляющее пост в системе
pub struct Post {
    // UUID поста в формате строки
    #[builder(try_setter,default = Uuid::new_v4())]
    #[serde(rename = "_id")]
    #[serde(with = "uuid_1::AsBinary")]
    pub id: Uuid,
    // Заголовок поста (обязательное поле)
    // Минимальная длина: 1 символ, максимальная: 255 символов
    #[builder(setter(into))]
    pub title: String,
    // Содержимое поста (обязательное поле)
    // Минимальная длина: 1 символ, максимальная: 4096 символов
    #[builder(setter(into))]
    pub content: String,
    // Идентификатор фотофайла в Telegram (если прикреплено)
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tg_photo_file_id: Option<String>,
    // Идентификатор фотофайла в VK (если прикреплено)
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vk_photo_file_id: Option<String>,
    // Идентификатор видеофайла в Telegram (если прикреплено)
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tg_video_file_id: Option<String>,
    // Идентификатор видеофайла в VK (если прикреплено)
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vk_video_file_id: Option<String>,
    // Текущий статус поста
    #[builder(try_setter, setter(into), default)]
    pub status: Status,
    // Дата и время создания поста
    #[builder(default = Utc::now())]
    #[serde(with = "datetime::FromChrono04DateTime")]
    pub created_at: DateTime<Utc>,
    // Запланированное время публикации (для отложенных постов)
    #[builder(default)]
    #[serde(
        serialize_with = "serialize_option_datetime",
        deserialize_with = "deserialize_option_datetime"
    )]
    pub publish_datetime: Option<DateTime<Utc>>,
    // Идентификатор автора поста (UUID пользователя)
    #[builder(try_setter, setter(into))]
    #[serde(with = "uuid_1::AsBinary")]
    pub author_id: Uuid,
}
impl Post {
    pub fn builder() -> PostBuilder {
        PostBuilder::default()
    }
}
impl PostBuilder {
    fn validate(&self) -> Result<(), String> {
        if let Some(title) = self.title.as_ref() {
            if title.len() < 1 || title.len() > 255 {
                return Err(String::from("wrong title"));
            }
        }
        if let Some(content) = self.content.as_ref() {
            if content.len() < 1 || content.len() > 4096 {
                return Err(String::from("wrong content"));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
// Статусы поста
pub enum Status {
    // Черновик
    #[default]
    Draft,
    // Ожидает публикации
    Pending,
    // Опубликован
    Published,
    // Отменен/Заброшен
    Abandoned,
}
impl TryFrom<i32> for Status {
    type Error = anyhow::Error;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Draft),
            1 => Ok(Self::Pending),
            2 => Ok(Self::Published),
            3 => Ok(Self::Abandoned),
            _ => Err(anyhow!("Invalid role value: {value}")),
        }
    }
}

impl From<Status> for i32 {
    fn from(status: Status) -> Self {
        status as i32
    }
}
impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl From<String> for Status {
    fn from(value: String) -> Self {
        if Status::Pending.to_string() == value {
            Self::Pending
        } else if Status::Published.to_string() == value {
            Self::Published
        } else if Status::Abandoned.to_string() == value {
            Self::Abandoned
        } else {
            Self::default()
        }
    }
}
// Ответ на запрос списка постов
pub struct ListPostsResult {
    // Список постов
    pub posts: Vec<Post>,
    // Общее количество постов
    pub total_count: u32,
    // Текущая страница
    pub current_page: u32,
    // Общее количество страниц
    pub total_pages: u32,
}
fn serialize_option_datetime<S>(
    datetime: &Option<DateTime<Utc>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match datetime {
        Some(dt) => datetime::FromChrono04DateTime::serialize(dt, serializer),
        None => serializer.serialize_none(),
    }
}

fn deserialize_option_datetime<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(transparent)]
    struct Helper(#[serde(with = "datetime::FromChrono04DateTime")] DateTime<Utc>);

    let opt: Option<Helper> = Option::deserialize(deserializer)?;
    Ok(opt.map(|h| h.0))
}
