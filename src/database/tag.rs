use std::option::Option::Some;

use futures::TryStreamExt;
use mongodb::{bson::doc, Client, options::ClientOptions};
use mongodb::options::FindOneOptions;

use crate::database::user::get_user_by_id;
use crate::dto::tag_dto::{CreateTag, TagAdmin, TagList, TagPage, UpdateTag};
use crate::dto::user_dto::SmallAccount;
use crate::error::ErrorMessage;
use crate::model::tag::Tag;

async fn connect() -> mongodb::Collection<Tag> {
    let mut conn = ClientOptions::parse("mongodb://admin:Lj6kuxGJh&k8CaN6UgsQF+aDVkQF3Wn7hdSeXke@localhost:27017/").await.unwrap();
    conn.app_name = Some("My App".to_string());
    let client: Client = Client::with_options(conn).unwrap();
    let db = client.database("test");
    db.collection::<Tag>("tag")
}

pub async fn find_one_id(id: i32) -> Tag {
    let conn = connect().await;
    let cursor = conn.find_one(doc! {"_id":id}, None).await.unwrap();
    cursor.unwrap()
}

pub async fn get_tag(user_id: Option<i32>, value: String) -> Result<TagPage, ErrorMessage> {
    let col = connect().await;
    let tag = col.find_one(doc! {"value":value}, None).await.unwrap();

    match tag {
        None => { return Err(ErrorMessage::NotFound); }
        Some(t) => {
            match user_id {
                None => {
                    let mut users: Vec<SmallAccount> = vec![];
                    for id in t.moderator.to_owned() {
                        let user = get_user_by_id(id).await.unwrap();
                        users.push(SmallAccount::from(user));
                    }
                    let mut tag_result = TagPage::from(t);
                    tag_result.moderator = users;
                    Ok(tag_result)
                }
                Some(id) => {
                    let check_follow = get_user_by_id(id).await.unwrap().followed_tag;
                    let mut users: Vec<SmallAccount> = vec![];
                    for id in t.moderator.to_owned() {
                        let user = get_user_by_id(id).await.unwrap();
                        users.push(SmallAccount::from(user));
                    }
                    let mut tag_result = TagPage::from(t.to_owned());
                    if check_follow.contains(&t.id) { tag_result.saved = true }
                    tag_result.moderator = users;
                    Ok(tag_result)
                }
            }
        }
    }
}


pub async fn get_tags(user_id: Option<i32>) -> Vec<TagList> {
    return match user_id {
        None => {
            let mut rs: Vec<TagList> = vec![];
            let col = connect().await;
            let mut cursor = col.find(None, None).await.unwrap();
            while let Some(tag) = cursor.try_next().await.unwrap() {
                rs.push(TagList::from(tag));
            }
            rs
        }
        Some(id) => {
            let mut rs: Vec<TagList> = vec![];
            let col = connect().await;
            let mut cursor = col.find(None, None).await.unwrap();
            let user = get_user_by_id(id).await.unwrap().followed_tag;
            while let Some(tag) = cursor.try_next().await.unwrap() {
                let mut re_tag = TagList::from(tag.to_owned());
                if user.contains(&tag.id) {
                    re_tag.saved = true;
                }
                rs.push(re_tag);
            }
            rs
        }
    };
}

pub async fn find_list_followed_tag(tag_list: &Vec<i32>) -> Vec<TagList> {
    let mut result: Vec<TagList> = vec![];
    let col = connect().await;
    let query = doc! {
        "_id":{
            "$in":tag_list
        }
    };
    let mut cursor = col.find(query, None).await.unwrap();
    while let Some(tag) = cursor.try_next().await.unwrap() {
        result.push(TagList::from(tag));
    }
    result
}

pub async fn tags() -> Vec<TagAdmin> {
    let mut res = Vec::new();
    let col = connect().await;
    let mut cursor = col.find(None, None).await.unwrap();
    while let Some(tag) = cursor.try_next().await.unwrap() {
        res.push(TagAdmin::from(tag))
    }
    return res;
}

pub async fn create_tag(tag_create: CreateTag) -> Result<Vec<TagAdmin>, ErrorMessage> {
    let col = connect().await;
    let sort = FindOneOptions::builder().sort(doc! {"_id":-1}).build();
    let last_tag = col.find_one(None, sort).await.unwrap();
    let duplicate = col.find_one(doc! {"value":&tag_create.value}, None).await.unwrap();
    match duplicate {
        None => {
            let tag = Tag {
                id: if last_tag.is_some() { last_tag.unwrap().id + 1 } else { 1 },
                value: tag_create.value,
                desc: tag_create.desc,
                color: tag_create.color,
                image: tag_create.image,
                post: 0,
                moderator: vec![],
            };
            match col.insert_one(tag, None).await{
                Ok(_) => {}
                Err(_) => {return Err(ErrorMessage::ServerError)}
            }
            Ok(tags().await)
        }
        Some(_) => { Err(ErrorMessage::Duplicate) }
    }
}

pub async fn update(tag_update: UpdateTag) -> Result<Vec<TagAdmin>, ErrorMessage> {
    let col = connect().await;
    let tag_opt = col.find_one(doc! {"_id":&tag_update.id}, None).await.unwrap();
    return match tag_opt {
        None => { Err(ErrorMessage::NotFound) }
        Some(mut tag) => {
            if tag_update.value.is_some() {
                if col.find_one(doc! {"value":&tag_update.value}, None).await.unwrap().is_some() {
                    return Err(ErrorMessage::Duplicate);
                } else {
                    tag.value = tag_update.value.unwrap()
                }
            }
            if tag_update.desc.is_some() { tag.desc = tag_update.desc.unwrap() }
            if tag_update.color.is_some() { tag.color = tag_update.color.unwrap() }
            if tag_update.image.is_some() { tag.image = tag_update.image.unwrap() }
            match col.replace_one(doc! {"_id":&tag.id}, tag, None).await {
                Ok(_) => {}
                Err(_) => { return Err(ErrorMessage::ServerError); }
            }
            Ok(tags().await)
        }
    };
}