use std::fmt::Display;

use bson::serde_helpers::{datetime, uuid_1};
use chrono::{DateTime, Utc};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
#[builder(build_fn(validate = "Self::validate"))]
pub struct User {
    /// UUID пользователя в формате строки (версия 4)
    #[builder(try_setter,default = Uuid::new_v4())]
    #[serde(rename = "_id")]
    #[serde(with = "uuid_1::AsBinary")]
    pub id: Uuid,

    /// Идентификатор пользователя в Telegram
    pub telegram_id: i64,

    /// Основное имя пользователя
    #[builder(setter(into))]
    pub first_name: String,

    /// Фамилия пользователя (если доступна)
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,

    /// Юзернейм в Telegram (если установлен)
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,

    /// Предпочитаемый язык пользователя
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language_code: Option<String>,

    /// Текущая роль пользователя в системе
    #[builder(try_setter, setter(into), default)]
    pub role: Role,

    /// Дата и время создания записи пользователя
    #[builder(setter(custom), default = Utc::now())]
    #[serde(with = "datetime::FromChrono04DateTime")]
    pub created_at: DateTime<Utc>,

    /// Дата и время последнего обновления данных
    #[builder(setter(custom), default = Utc::now())]
    #[serde(with = "datetime::FromChrono04DateTime")]
    pub updated_at: DateTime<Utc>,

    /// Дата и время последней активности пользователя
    #[builder(setter(custom), default = Utc::now())]
    #[serde(with = "datetime::FromChrono04DateTime")]
    pub last_activity: DateTime<Utc>,
}
impl User {
    pub fn builder() -> UserBuilder {
        UserBuilder::default()
    }
}
impl UserBuilder {
    pub fn validate(&self) -> Result<(), String> {
        if let Some(tid) = self.telegram_id {
            if tid < 0 {
                return Err(String::from("telegram id must be greater than 0"));
            }
        }
        if let Some(firstname) = self.first_name.as_ref() {
            if firstname.len() < 1 || firstname.len() > 64 {
                return Err(String::from("wrong first name"));
            }
        }

        if let Some(lastname) = self.last_name.as_ref().and_then(|l| l.as_ref()) {
            if lastname.len() < 1 || lastname.len() > 64 {
                return Err(String::from("wrong last name"));
            }
        }

        if let Some(un) = self.username.as_ref().and_then(|u| u.as_ref()) {
            if let Ok(re) = regex::Regex::new(r"^[a-zA-Z0-9_]{5,32}$") {
                if !re.is_match(un) {
                    return Err(String::from("wrong username"));
                }
            }
        }

        if let Some(lc) = self.language_code.as_ref().and_then(|c| c.as_ref()) {
            if let Ok(re) = regex::Regex::new(r"^[a-z]{2}$") {
                if !re.is_match(lc) {
                    return Err(String::from("wrong language code"));
                }
            }
        }
        Ok(())
    }
    pub fn created_at(&mut self, seconds: i64, nanos: i32) -> &mut Self {
        self.created_at = DateTime::from_timestamp(seconds, nanos as u32);
        self
    }
    pub fn updated_at(&mut self, seconds: i64, nanos: i32) -> &mut Self {
        self.updated_at = DateTime::from_timestamp(seconds, nanos as u32);
        self
    }
    pub fn last_activity(&mut self, seconds: i64, nanos: i32) -> &mut Self {
        self.last_activity = DateTime::from_timestamp(seconds, nanos as u32);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    /// Роль не определена (значение по умолчанию)
    Guest,
    /// Редактор - может создавать и управлять своим контентом
    Editor,
    /// Администратор - полный доступ к системе
    Admin,
}

impl Default for Role {
    fn default() -> Self {
        Self::Guest
    }
}

impl TryFrom<i32> for Role {
    type Error = String;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Guest),
            1 => Ok(Self::Editor),
            2 => Ok(Self::Admin),
            _ => Err(format!("Invalid role value: {value}")),
        }
    }
}

impl From<Role> for i32 {
    fn from(role: Role) -> Self {
        role as i32
    }
}
impl Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl From<String> for Role {
    fn from(value: String) -> Self {
        if Role::Admin.to_string() == value {
            Self::Admin
        } else if Role::Editor.to_string() == value {
            Self::Editor
        } else {
            Self::default()
        }
    }
}
// Ответ на запрос списка пользователей
pub struct ListUsersResult {
    // Список пользователей
    pub users: Vec<User>,
    // Общее количество пользователей
    pub total_count: u32,
    // Текущая страница
    pub current_page: u32,
    // Общее количество страниц
    pub total_pages: u32,
}
