use chrono::{DateTime, Utc};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
#[builder(build_fn(validate = "Self::validate"))]
pub struct User {
    /// UUID пользователя в формате строки (версия 4)
    #[builder(try_setter,default = Uuid::new_v4())]
    pub id: Uuid,

    /// Идентификатор пользователя в Telegram
    pub telegram_id: i64,

    /// Основное имя пользователя
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
    pub role: UserRole,

    /// Дата и время создания записи пользователя
    #[builder(setter(custom), default = Utc::now())]
    pub created_at: DateTime<Utc>,

    /// Дата и время последнего обновления данных
    #[builder(setter(custom), default = Utc::now())]
    pub updated_at: DateTime<Utc>,

    /// Дата и время последней активности пользователя
    #[builder(setter(custom), default = Utc::now())]
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
pub enum UserRole {
    /// Роль не определена (значение по умолчанию)
    Guest = 0,

    /// Редактор - может создавать и управлять своим контентом
    Editor = 1,

    /// Администратор - полный доступ к системе
    Admin = 2,
}

impl Default for UserRole {
    fn default() -> Self {
        Self::Guest
    }
}

impl TryFrom<i32> for UserRole {
    type Error = String;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Guest),
            1 => Ok(Self::Editor),
            2 => Ok(Self::Admin),
            _ => Err(format!("Invalid role value: {}", value)),
        }
    }
}

impl From<UserRole> for i32 {
    fn from(role: UserRole) -> Self {
        role as i32
    }
}
