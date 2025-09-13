use std::{fmt::Display, str::FromStr};

use anyhow::anyhow;
use teloxide::types::{KeyboardButton, KeyboardMarkup};

const USERS: &str = "👨🏻‍💻 Пользователи";
const CREATE_POST: &str = "➕ Новый пост";
const DRAFTS: &str = "✍️ Черновики";
const PENDING: &str = "⌛ В очереди";
const PUBLISHED: &str = "✔️ Опубликованные";
const REQUEST_ACCESS: &str = "🙏 Запросить доступ";

#[derive(Clone)]
pub enum TextCommand {
    Users,
    CreatePost,
    Drafts,
    Pending,
    Published,
    RequestAccess,
}
impl TextCommand {
    pub fn admin_keyboard() -> KeyboardMarkup {
        KeyboardMarkup::default()
            .append_row(vec![TextCommand::Users.into()])
            .append_row(vec![
                TextCommand::CreatePost.into(),
                TextCommand::Drafts.into(),
            ])
            .append_row(vec![
                TextCommand::Pending.into(),
                TextCommand::Published.into(),
            ])
            .resize_keyboard()
    }
    pub fn editor_keyboard() -> KeyboardMarkup {
        KeyboardMarkup::default()
            .append_row(vec![
                TextCommand::CreatePost.into(),
                TextCommand::Drafts.into(),
            ])
            .append_row(vec![
                TextCommand::Pending.into(),
                TextCommand::Published.into(),
            ])
            .resize_keyboard()
    }
    pub fn guest_keyboard() -> KeyboardMarkup {
        KeyboardMarkup::default()
            .append_row(vec![TextCommand::RequestAccess.into()])
            .resize_keyboard()
    }
}
impl FromStr for TextCommand {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            USERS => Ok(Self::Users),
            CREATE_POST => Ok(Self::CreatePost),
            DRAFTS => Ok(Self::Drafts),
            PENDING => Ok(Self::Pending),
            PUBLISHED => Ok(Self::Published),
            REQUEST_ACCESS => Ok(Self::RequestAccess),
            _ => Err(anyhow!("not a text command")),
        }
    }
}
impl Display for TextCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            TextCommand::Users => USERS,
            TextCommand::CreatePost => CREATE_POST,
            TextCommand::Drafts => DRAFTS,
            TextCommand::Pending => PENDING,
            TextCommand::Published => PUBLISHED,
            TextCommand::RequestAccess => REQUEST_ACCESS,
        };
        write!(f, "{s}")
    }
}
impl From<TextCommand> for KeyboardButton {
    fn from(value: TextCommand) -> Self {
        KeyboardButton::new(value.to_string())
    }
}
