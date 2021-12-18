use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum Status {
    Published,
    Draft,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Comment {
    #[serde(rename = "_id")]
    pub id: i32,
    pub content: String,
    #[serde(rename = "userUserName")]
    pub user_username: String,
    #[serde(rename = "userAvatar")]
    pub user_avatar: String,
    #[serde(rename = "userName")]
    pub user_name: String,
    #[serde(rename = "createdAt")]
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "updatedAt")]
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub interact: i32,
    #[serde(rename = "parentId")]
    pub parent_id: i32,
    #[serde(rename = "interactList")]
    pub interact_list: Vec<i32>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Post {
    #[serde(rename = "_id")]
    pub id: i32,
    #[serde(rename = "userUserName")]
    pub user_username: String,
    #[serde(rename = "userAvatar")]
    pub user_avatar: String,
    #[serde(rename = "userName")]
    pub user_name: String,
    pub slug: String,
    pub banner: String,
    pub title: String,
    pub content: String,
    #[serde(rename = "createdAt")]
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    #[serde(rename = "updatedAt")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub status: Status,
    pub tag: Vec<String>,
    pub comment: Vec<Comment>,
    #[serde(rename = "commentCount")]
    pub comment_count: i32,
    #[serde(rename = "reactionCount")]
    pub reaction_count: i32,
    #[serde(rename = "reactionList")]
    pub reaction_list: Vec<i32>,
    #[serde(rename = "commentList")]
    pub comment_list: Vec<i32>,
    #[serde(rename = "savedByUser")]
    pub saved_by_user: Vec<i32>,
}
