use chrono::Utc;
use serde::{Deserialize, Serialize};
use crate::dto::user_dto::CreateAccount;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum Status {
    Activated,
    Banned,
    Pending,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Account {
    #[serde(rename = "_id")]
    pub id: i32,
    pub name: String,
    pub username: String,
    #[serde(rename = "schoolEmail")]
    pub school_email: String,
    #[serde(rename = "privateEmail")]
    pub private_email: String,
    pub bio: String,
    pub password: String,
    pub avatar: String,
    pub admin: bool,
    pub website: String,
    #[serde(rename = "lastAccess")]
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub last_access: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "createdAt")]
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "updatedAt")]
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub status: Status,
    #[serde(rename = "followedTag")]
    pub followed_tag: Vec<i32>,
    #[serde(rename = "readingList")]
    pub reading_list: Vec<i32>,
    #[serde(rename = "followedUser")]
    pub followed_user: Vec<i32>,
}


impl From<CreateAccount> for Account {
    fn from(acc: CreateAccount) -> Self {
        Account {
            id: 0,
            name: acc.name,
            username: acc.username,
            school_email: acc.school_email,
            private_email: acc.private_email,
            bio: if acc.bio.is_some() { acc.bio.unwrap() } else { "".to_string() },
            password: acc.password,
            avatar: if acc.avatar.is_some() { acc.avatar.unwrap() } else { "".to_string() },
            admin: false,
            website: if acc.website.is_some() { acc.website.unwrap() } else { "".to_string() },
            last_access: Utc::now(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            status: Status::Activated,
            followed_tag: vec![],
            reading_list: vec![],
            followed_user: vec![],
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Claim {
    pub sub: i32,
    pub exp: usize,
}
