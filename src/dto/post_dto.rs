use crate::model::post::{Comment, Post, Status};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct CreatePost {
    pub banner: String,
    pub title: String,
    pub content: String,
    pub tag: [String; 3],
    pub status: Status,
}

#[derive(Serialize, Deserialize)]
pub struct UpdatePost {
    pub slug: String,
    pub banner: Option<String>,
    pub title: Option<String>,
    pub content: Option<String>,
    pub tag: Option<[String; 3]>,
    pub status: Option<Status>,
}


#[derive(Deserialize, Serialize)]
pub struct CreateComment {
    pub content: String,
    pub slug: String,
    #[serde(rename = "parentId")]
    pub parent_id: i32,
}

#[derive(Deserialize, Serialize)]
pub struct UpdateComment {
    pub id: i32,
    pub content: String,
    #[serde(rename = "slug")]
    pub slug_post: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Index {
    pub id: i32,
    #[serde(rename = "userUsername")]
    pub user_username: String,
    #[serde(rename = "userAvatar")]
    pub user_avatar: String,
    #[serde(rename = "userName")]
    pub user_name: String,
    pub slug: String,
    pub title: String,
    #[serde(rename = "createdAt")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub status: Status,
    #[serde(rename = "tagList")]
    pub tag_list: [String; 3],
    #[serde(rename = "commentCount")]
    pub comment_count: i32,
    #[serde(rename = "reactionCount")]
    pub reaction_count: i32,
    pub save: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CommentInfoPage {
    #[serde(rename = "slug")]
    pub post_slug: String,
    #[serde(rename = "postTitle")]
    pub post_title: String,
    #[serde(rename = "parentComment")]
    pub parent_comment: Option<CommentDetail>,
    #[serde(rename = "childComment")]
    pub child_comment: CommentDetail,
}


#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct CommentDetail {
    pub id: i32,
    pub content: String,
    #[serde(rename = "userUserName")]
    pub user_username: String,
    #[serde(rename = "userAvatar")]
    pub user_avatar: String,
    #[serde(rename = "userName")]
    pub user_name: String,
    #[serde(rename = "createdAt")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub interact: i32,
    #[serde(rename = "parentId")]
    pub parent_id: i32,
    pub interacted: bool,
}

impl From<Comment> for CommentDetail {
    fn from(com: Comment) -> Self {
        CommentDetail {
            id: com.id,
            content: com.content,
            user_username: com.user_username,
            user_avatar: com.user_avatar,
            user_name: com.user_name,
            created_at: com.created_at,
            updated_at: com.updated_at,
            interact: com.interact,
            parent_id: com.parent_id,
            interacted: false,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ShortPost {
    pub slug: String,
    pub title: String,
    pub status: Status,
}

impl From<Post> for ShortPost {
    fn from(p: Post) -> Self {
        ShortPost {
            slug: p.slug,
            title: p.title,
            status: p.status,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct PostDetail {
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
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub status: Status,
    pub tag: [String; 3],
    pub comment: Vec<PostDetailComment>,
    pub followed: bool,
    pub interacted: bool,
}

impl From<Post> for PostDetail {
    fn from(p: Post) -> Self {
        PostDetail {
            id: p.id,
            user_username: p.user_username,
            user_avatar: p.user_avatar,
            user_name: p.user_name,
            slug: p.slug,
            banner: p.banner,
            title: p.title,
            content: p.content,
            created_at: p.created_at,
            updated_at: p.updated_at,
            status: p.status,
            tag: p.tag,
            comment: vec![],
            followed: false,
            interacted: false,
        }
    }
}


#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct PostDetailComment {
    pub id: i32,
    pub content: String,
    #[serde(rename = "userUserName")]
    pub user_username: String,
    #[serde(rename = "userAvatar")]
    pub user_avatar: String,
    #[serde(rename = "userName")]
    pub user_name: String,
    #[serde(rename = "createdAt")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub interact: i32,
    #[serde(rename = "parentId")]
    pub parent_id: i32,
    pub interacted: bool,
}

impl From<Comment> for PostDetailComment {
    fn from(c: Comment) -> Self {
        PostDetailComment {
            id: c.id,
            content: c.content,
            user_username: c.user_username,
            user_avatar: c.user_avatar,
            user_name: c.user_name,
            created_at: c.created_at,
            updated_at: c.updated_at,
            interact: c.interact,
            parent_id: c.parent_id,
            interacted: false,
        }
    }
}