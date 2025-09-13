use std::{fmt::Display, str::FromStr};

use anyhow::anyhow;
use shared::models::Status;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use uuid::Uuid;

const MAKE_USER_EDITOR: &str = "⬆️ В редакторы";
const MAKE_USER_GUEST: &str = "⬇️ В гости";
const DELETE_USER: &str = "🗑️ Удалить пользователя";
const CANCEL: &str = "❌ Отмена";
const PUBLISH_NOW: &str = "Опубликовать";
const DELETE_POST: &str = "Удалить пост";
const SET_PUBLISH_DATE: &str = "Запланировать";
const POSTS_NEXT_PAGE: &str = "Следующая ⏭️";
const POSTS_PREVIOUS_PAGE: &str = "⏮️ Предыдущая";
const DRAFTS: &str = "Черновики";
const PENDING: &str = "В очереди";
const PUBLISHED: &str = "Опубликованные";

#[derive(Debug, Clone)]
pub enum MyCallback {
    Cancel,
    MakeUserEditor {
        id: i64,
    },
    MakeUserGuest {
        id: i64,
    },
    DeleteUser {
        id: i64,
    },
    PublishNow {
        id: Uuid,
    },
    DeletePost {
        id: Uuid,
    },
    SetPublishDate {
        id: Uuid,
    },
    PostsNextPage {
        author_id: i64,
        status: Status,
        page: u32,
    },
    PostsPreviousPage {
        author_id: i64,
        status: Status,
        page: u32,
    },
    Drafts {
        author_id: i64,
    },
    Pending {
        author_id: i64,
    },
    Published {
        author_id: i64,
    },
}
impl MyCallback {
    pub fn data(&self) -> String {
        match self {
            MyCallback::Cancel => self.to_string(),
            MyCallback::MakeUserEditor { id } => format!("{self}:{id}"),
            MyCallback::MakeUserGuest { id } => format!("{self}:{id}"),
            MyCallback::DeleteUser { id } => format!("{self}:{id}"),
            MyCallback::Drafts { author_id } => format!("{self}:{author_id}"),
            MyCallback::Pending { author_id } => format!("{self}:{author_id}"),
            MyCallback::Published { author_id } => format!("{self}:{author_id}"),
            MyCallback::PublishNow { id } => {
                let id = id.clone();
                format!("{self}:{id}")
            }
            MyCallback::DeletePost { id } => {
                let id = id.clone();
                format!("{self}:{id}")
            }
            MyCallback::SetPublishDate { id } => {
                let id = id.clone();
                format!("{self}:{id}")
            }
            MyCallback::PostsNextPage {
                author_id,
                status,
                page,
            } => format!("{self}:{author_id}:{status}:{page}"),
            MyCallback::PostsPreviousPage {
                author_id,
                status,
                page,
            } => format!("{self}:{author_id}:{status}:{page}"),
        }
    }
    pub fn guest_kb(id: i64) -> InlineKeyboardMarkup {
        InlineKeyboardMarkup::default()
            .append_row(vec![
                MyCallback::MakeUserEditor { id }.into(),
                MyCallback::DeleteUser { id }.into(),
            ])
            .append_row(vec![
                MyCallback::Drafts { author_id: id }.into(),
                MyCallback::Pending { author_id: id }.into(),
            ])
            .append_row(vec![MyCallback::Published { author_id: id }.into()])
    }
    pub fn editor_kb(id: i64) -> InlineKeyboardMarkup {
        InlineKeyboardMarkup::default()
            .append_row(vec![
                MyCallback::MakeUserGuest { id }.into(),
                MyCallback::DeleteUser { id }.into(),
            ])
            .append_row(vec![
                MyCallback::Drafts { author_id: id }.into(),
                MyCallback::Pending { author_id: id }.into(),
            ])
            .append_row(vec![MyCallback::Published { author_id: id }.into()])
    }
    pub fn not_published_kb(id: Uuid) -> InlineKeyboardMarkup {
        InlineKeyboardMarkup::default()
            .append_row(vec![
                MyCallback::PublishNow { id }.into(),
                MyCallback::SetPublishDate { id }.into(),
            ])
            .append_row(vec![MyCallback::DeletePost { id }.into()])
    }
    pub fn published_kb(id: Uuid) -> InlineKeyboardMarkup {
        InlineKeyboardMarkup::default().append_row(vec![MyCallback::DeletePost { id }.into()])
    }
    pub fn has_next_kb(author_id: i64, status: Status, page: u32) -> InlineKeyboardMarkup {
        InlineKeyboardMarkup::default().append_row(vec![
            MyCallback::Cancel.into(),
            MyCallback::PostsNextPage {
                author_id,
                status,
                page,
            }
            .into(),
        ])
    }
    pub fn has_previous_kb(author_id: i64, status: Status, page: u32) -> InlineKeyboardMarkup {
        InlineKeyboardMarkup::default().append_row(vec![
            MyCallback::PostsPreviousPage {
                author_id,
                status,
                page,
            }
            .into(),
            MyCallback::Cancel.into(),
        ])
    }
    pub fn has_previous_and_next_kb(
        author_id: i64,
        status: Status,
        current_page: u32,
    ) -> InlineKeyboardMarkup {
        let c = format!("{current_page}");
        InlineKeyboardMarkup::default()
            .append_row(vec![
                MyCallback::PostsPreviousPage {
                    author_id,
                    status,
                    page: current_page - 1,
                }
                .into(),
                InlineKeyboardButton::callback(&c, &c),
                MyCallback::PostsNextPage {
                    author_id,
                    status,
                    page: current_page + 1,
                }
                .into(),
            ])
            .append_row(vec![MyCallback::Cancel.into()])
    }
    pub fn cancel_button() -> InlineKeyboardMarkup {
        InlineKeyboardMarkup::default().append_row(vec![MyCallback::Cancel.into()])
    }
}
impl Display for MyCallback {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            MyCallback::MakeUserEditor { .. } => MAKE_USER_EDITOR,
            MyCallback::MakeUserGuest { .. } => MAKE_USER_GUEST,
            MyCallback::DeleteUser { .. } => DELETE_USER,
            MyCallback::Cancel => CANCEL,
            MyCallback::PublishNow { .. } => PUBLISH_NOW,
            MyCallback::DeletePost { .. } => DELETE_POST,
            MyCallback::SetPublishDate { .. } => SET_PUBLISH_DATE,
            MyCallback::PostsNextPage { .. } => POSTS_NEXT_PAGE,
            MyCallback::PostsPreviousPage { .. } => POSTS_PREVIOUS_PAGE,
            MyCallback::Drafts { .. } => DRAFTS,
            MyCallback::Pending { .. } => PENDING,
            MyCallback::Published { .. } => PUBLISHED,
        };
        write!(f, "{s}")
    }
}
impl From<MyCallback> for InlineKeyboardButton {
    fn from(value: MyCallback) -> Self {
        InlineKeyboardButton::callback(value.to_string(), value.data())
    }
}
impl FromStr for MyCallback {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == CANCEL {
            return Ok(Self::Cancel);
        }
        let (action, data) = s.split_once(':').ok_or(anyhow!("not a callback"))?;
        match action {
            MAKE_USER_EDITOR => {
                let id = data.parse()?;
                Ok(Self::MakeUserEditor { id })
            }
            MAKE_USER_GUEST => {
                let id = data.parse()?;
                Ok(Self::MakeUserGuest { id })
            }
            DELETE_USER => {
                let id = data.parse()?;
                Ok(Self::DeleteUser { id })
            }
            CANCEL => Ok(Self::Cancel),
            PUBLISH_NOW => {
                let id = data.parse()?;
                Ok(Self::PublishNow { id })
            }
            DELETE_POST => {
                let id = data.parse()?;
                Ok(Self::DeletePost { id })
            }
            SET_PUBLISH_DATE => {
                let id = data.parse()?;
                Ok(Self::SetPublishDate { id })
            }
            DRAFTS => {
                let author_id = data.parse()?;
                Ok(Self::Drafts { author_id })
            }
            PENDING => {
                let author_id = data.parse()?;
                Ok(Self::Pending { author_id })
            }
            PUBLISHED => {
                let author_id = data.parse()?;
                Ok(Self::Published { author_id })
            }
            POSTS_NEXT_PAGE => {
                let s = data.split(':').collect::<Vec<_>>();
                if s.len() != 3 {
                    return Err(anyhow!("not a callback"));
                }
                let author_id = s[0].parse()?;
                let status = s[1].to_string().into();
                let page = s[2].parse()?;
                Ok(Self::PostsNextPage {
                    author_id,
                    status,
                    page,
                })
            }
            POSTS_PREVIOUS_PAGE => {
                let s = data.split(':').collect::<Vec<_>>();
                if s.len() != 3 {
                    return Err(anyhow!("not a callback"));
                }
                let author_id = s[0].parse()?;
                let status = s[1].to_string().into();
                let page = s[2].parse()?;
                Ok(Self::PostsPreviousPage {
                    author_id,
                    status,
                    page,
                })
            }
            _ => Err(anyhow!("not a callback")),
        }
    }
}
