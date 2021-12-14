use serde::{Serialize, Deserialize};
use crate::model::user::{Account};

#[derive(Deserialize, Serialize)]
pub struct AccountStore {
    pub id: i32,
    pub name: String,
    pub username: String,
    #[serde(rename = "schoolEmail")]
    pub school_email: String,
    #[serde(rename = "privateEmail")]
    pub private_email: String,
    pub bio: String,
    pub avatar: String,
    pub admin: bool,
    pub website: String,
    pub token: String,
}

impl From<Account> for AccountStore {
    fn from(info: Account) -> Self {
        AccountStore {
            id: info.id,
            name: info.name,
            username: info.username,
            school_email: info.school_email,
            private_email: info.private_email,
            bio: info.bio,
            avatar: info.avatar,
            admin: info.admin,
            website: info.website,
            token: "".to_string(),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct ShowAccountAdmin {
    name: String,
    username: String,
    school_email: String,
    email: String,
    avatar: String,
    admin: bool,
}

impl From<Account> for ShowAccountAdmin {
    fn from(acc: Account) -> Self {
        ShowAccountAdmin {
            name: acc.name,
            username: acc.username,
            school_email: acc.school_email,
            email: acc.private_email,
            avatar: acc.avatar,
            admin: acc.admin,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct CreateAccount {
    pub id: Option<i32>,
    pub name: String,
    pub username: String,
    pub school_email: String,
    pub private_email: String,
    pub password: String,
    #[serde(rename = "followedTag")]
    pub followed_tag: Option<Vec<i32>>,
}

#[derive(Deserialize, Serialize)]
pub struct UserPage {
    pub name: String,
    pub username: String,
    pub bio: String,
    pub avatar: String,
    pub website: String,
    #[serde(rename = "createdAt")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub followed: bool,
}

impl From<Account> for UserPage {
    fn from(acc: Account) -> Self {
        UserPage {
            name: acc.name,
            username: acc.username,
            bio: acc.bio,
            avatar: acc.avatar,
            website: acc.website,
            created_at: acc.created_at,
            followed: false,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct SmallAccount {
    #[serde(rename = "userUserName")]
    pub username: String,
    #[serde(rename = "userAvatar")]
    pub avatar: String,
    #[serde(rename = "userName")]
    pub name: String,
    pub followed: bool,
}

impl From<Account> for SmallAccount {
    fn from(acc: Account) -> Self {
        SmallAccount {
            username: acc.username,
            avatar: acc.avatar,
            name: acc.name,
            followed: false,
        }
    }
}