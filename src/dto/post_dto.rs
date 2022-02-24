use serde::{Deserialize, Serialize};

use crate::dto::tag_dto::ShortTag;
use crate::dto::user_dto::PostDetailUser;
use crate::model::post::{Comment, Post, Status};

#[derive(Serialize, Deserialize)]
pub struct CreatePost {
    pub banner: Option<String>,
    pub title: String,
    pub content: String,
    pub tag: Vec<String>,
    pub status: Status,
}

#[derive(Serialize, Deserialize)]
pub struct UpdatePost {
    pub slug: String,
    pub banner: Option<String>,
    pub title: Option<String>,
    pub content: Option<String>,
    pub tag: Option<Vec<String>>,
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
    pub banner: String,
    pub title: String,
    #[serde(rename = "createdAt")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub status: Status,
    #[serde(rename = "tagList")]
    pub tag_list: Vec<String>,
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
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "updatedAt")]
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
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
    pub banner: Option<String>,
    pub title: String,
    pub content: String,
    #[serde(rename = "createdAt")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub status: Status,
    pub tag: Vec<String>,
    pub comment: Vec<PostDetailComment>,
    #[serde(rename = "interactCount")]
    pub interact_count: i32,
    #[serde(rename = "commentCount")]
    pub comment_count: i32,
    #[serde(rename = "savedCount")]
    pub saved_count: i32,
    pub followed: bool,
    pub interacted: bool,
    pub saved: bool,
}

impl From<Post> for PostDetail {
    fn from(post: Post) -> Self {
        let mut vec_comment: Vec<PostDetailComment> = Vec::new();
        for p in post.comment.to_owned() {
            vec_comment.push(PostDetailComment::from(p))
        }
        PostDetail {
            id: post.id,
            user_username: post.user_username,
            user_avatar: post.user_avatar,
            user_name: post.user_name,
            slug: post.slug,
            banner: post.banner,
            title: post.title,
            content: post.content,
            created_at: post.created_at,
            updated_at: post.updated_at,
            status: post.status,
            tag: post.tag,
            comment: vec_comment,
            interact_count: post.reaction_count,
            comment_count: post.comment_count,
            saved_count: post.saved_by_user.len() as i32,
            followed: false,
            interacted: false,
            saved: false,
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

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ShortPostAdmin {
    pub slug: String,
    pub title: String,
    pub username: String,
    pub interact: i32,
    pub banner: Option<String>,
    pub comment: i32,
    #[serde(rename = "createdAt")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub tag: Vec<ShortTag>,
}

impl From<Post> for ShortPostAdmin {
    fn from(post: Post) -> Self {
        ShortPostAdmin {
            slug: post.slug,
            title: post.title,
            username: post.user_username,
            interact: post.reaction_count,
            banner: post.banner,
            comment: post.comment_count,
            created_at: post.created_at,
            updated_at: post.updated_at,
            tag: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct MorePost {
    pub slug: String,
    pub title: String,
    pub tag: Vec<String>,
}

impl From<Post> for MorePost {
    fn from(post: Post) -> Self {
        MorePost {
            slug: post.slug,
            title: post.title,
            tag: post.tag,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct PostDetailPage {
    #[serde(rename = "morePost")]
    pub more_post: Vec<MorePost>,
    #[serde(rename = "userInfo")]
    pub user_info: PostDetailUser,
    #[serde(rename = "postDetail")]
    pub post_detail: PostDetail,
}