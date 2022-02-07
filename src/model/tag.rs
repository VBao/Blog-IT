use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize,Clone)]
pub enum Type {
    Category,
    Tag,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Tag {
    #[serde(rename = "_id")]
    pub id: i32,
    pub value: String,
    pub desc: String,
    pub color: String,
    pub image: String,
    pub post: i32,
    #[serde(rename = "type")]
    pub types: Type,
    pub moderator: Vec<i32>,
}