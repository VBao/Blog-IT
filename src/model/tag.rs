use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize,Clone)]
pub struct Tag {
    #[serde(rename = "_id")]
    pub id: i32,
    pub value: String,
    pub desc: String,
    pub color: String,
    pub image: String,
    pub post: i32,
    pub moderator: Vec<i32>,
}