use serde::{Deserialize, Serialize};

use crate::dto::user_dto::SmallAccount;
use crate::model::tag::Tag;

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
            post_count: tag.post,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct CreateTag {
    pub value: String,
    pub desc: String,
    pub color: String,
    pub image: String,
}

#[derive(Deserialize, Serialize)]
pub struct UpdateTag {
    pub id: i32,
    pub value: Option<String>,
    pub desc: Option<String>,
    pub color: Option<String>,
    pub image: Option<String>,
}