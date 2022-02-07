use serde::{Deserialize, Serialize};

use crate::dto::user_dto::SmallAccount;
use crate::model::tag::{Tag, Type};

#[derive(Deserialize, Serialize)]
pub struct TagPage {
    pub value: String,
    pub desc: String,
    pub color: String,
    pub image: String,
    pub saved: bool,
    pub moderator: Vec<SmallAccount>,
}

impl From<Tag> for TagPage {
    fn from(tag: Tag) -> Self {
        TagPage {
            value: tag.value,
            desc: tag.desc,
            color: tag.color,
            image: tag.image,
            saved: false,
            moderator: vec![],
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct TagList {
    pub value: String,
    pub desc: String,
    pub color: String,
    pub image: String,
    pub saved: bool,
}

impl From<Tag> for TagList {
    fn from(t: Tag) -> Self {
        TagList {
            value: t.value,
            desc: t.desc,
            color: t.color,
            image: t.image,
            saved: false,
        }
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct TagAdmin {
    pub id: i32,
    pub value: String,
    pub desc: String,
    pub color: String,
    pub image: String,
    #[serde(rename = "type")]
    pub types: Type,
    #[serde(rename = "createdAt")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "postCount")]
    pub post_count: i32,
    // pub moderator: Vec<String>,
}

impl From<Tag> for TagAdmin {
    fn from(tag: Tag) -> Self {
        TagAdmin {
            id: tag.id,
            value: tag.value,
            desc: tag.desc,
            color: tag.color,
            image: tag.image,
            types: tag.types,
            created_at: tag.created_at,
            updated_at: tag.updated_at,
            post_count: tag.post,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct CreateTag {
    pub value: String,
    pub desc: String,
    #[serde(rename = "type")]
    pub types: Type,
    pub color: String,
    pub image: String,
}

#[derive(Deserialize, Serialize)]
pub struct UpdateTag {
    pub id: i32,
    pub value: Option<String>,
    pub desc: Option<String>,
    #[serde(rename = "type")]
    pub types: Type,
    pub color: Option<String>,
    pub image: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct ShortTag {
    pub value: String,
    pub image: String,
}

impl From<Tag> for ShortTag {
    fn from(tag: Tag) -> Self {
        ShortTag {
            value: tag.value,
            image: tag.image,
        }
    }
}

#[derive(Default, Deserialize, Serialize)]
pub struct IndexTag {
    pub tag: Vec<ShortTag>,
    pub category: Vec<ShortTag>,
}