use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Type {
    Category,
    Tag,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Tag {
    #[serde(rename = "_id")]
    pub id: i32,
    pub value: String,
    pub desc: String,
    pub color: String,
    pub image: String,
    pub post: i32,
    #[serde(rename = "createdAt")]
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    #[serde(rename = "updatedAt")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "type")]
    pub types: Type,
    pub moderator: Vec<i32>,
}