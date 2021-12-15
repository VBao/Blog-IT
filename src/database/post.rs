use std::borrow::{Borrow};
use std::option::Option::Some;
use std::prelude::rust_2021::Option;
use chrono::{DateTime, Utc};
use futures::{TryStreamExt};
use mongodb::{bson::{doc}, options::ClientOptions, Client, Collection, Cursor};
use mongodb::options::{FindOneOptions, FindOptions};
use rand::distributions::Alphanumeric;
use rand::{Rng, thread_rng};
use slug::slugify;
use crate::constant::MONGODB_URL;
use crate::database::tag;
use crate::model::post::*;
use crate::database::user::{connect as connect_user, get_user_by_id};
use crate::model::user::Account;
use crate::dto::post_dto::{CommentDetail, CommentInfoPage, CreateComment, CreatePost, Index, PostDetail, PostDetailComment, UpdateComment, UpdatePost};
use crate::dto::tag_dto::TagList;
use crate::error::ErrorMessage;
use crate::model::tag::Tag;

async fn connection_post() -> Collection<Post> {
    let mut conn = ClientOptions::parse(MONGODB_URL).await.unwrap();
    conn.app_name = Some("My App".to_string());
    let client = Client::with_options(conn).unwrap();
    let db = client.database("test");
    db.collection::<Post>("post")
}

async fn slug_generate(title: &String, username: &String) -> String {
    let col = connection_post().await;
    let mut slug = slugify(title.to_owned()) + &*username;
    let mut rand_string: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect();
    slug.push_str(&*rand_string);
    while col.find_one(doc! {"slug":slug.to_owned()}, None).await.unwrap().is_some() {
        rand_string = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(30)
            .map(char::from)
            .collect();
        slug.push_str(&*rand_string);
    };
    return slug;
}

async fn map_index(mut cursor: Cursor<Post>, account: &Option<Account>) -> Vec<Index> {
    let mut rs: Vec<Index> = vec![];
    while let Some(post) = cursor.try_next().await.unwrap() {
        rs.push(Index {
            id: post.id,
            user_username: post.user_username,
            user_avatar: post.user_avatar,
            user_name: post.user_name,
            slug: post.slug,
            title: post.title,
            created_at: post.created_at,
            updated_at: post.updated_at,
            status: Status::Published,
            tag_list: post.tag,
            comment_count: post.comment_count,
            reaction_count: post.reaction_count,
            save: if account.is_some() {
                account.as_ref().unwrap().reading_list.contains(&post.id)
            } else {
                false
            },
        })
    };
    return rs;
}

pub async fn connection_tag() -> Collection<Tag> {
    let mut conn = ClientOptions::parse("mongodb://admin:ChiBao07032001@localhost:27017/").await.unwrap();
    conn.app_name = Some("My App".to_string());
    let client = Client::with_options(conn).unwrap();
    let db = client.database("test");
    db.collection::<Tag>("tag")
}

pub async fn create_post(account: Account, create: CreatePost) -> PostDetail {
    let col = connection_post().await;
    let now: DateTime<Utc> = Utc::now();
    let sort = FindOneOptions::builder().sort(doc! {"_id":-1}).build();
    let last_post = col.borrow().find_one(None, sort).await.unwrap();
    let slug = slug_generate(&create.title, &account.username).await;
    let post = Post {
        id: last_post.unwrap().id + 1,
        user_username: account.username,
        user_avatar: account.avatar,
        user_name: account.name,
        slug,
        banner: create.banner.to_string(),
        title: create.title.to_string(),
        content: create.content.to_string(),
        created_at: now.into(),
        updated_at: now.into(),
        status: create.status,
        tag: create.tag,
        comment: vec![],
        comment_count: 0,
        reaction_count: 0,
        saved_count: 0,
        reaction_list: vec![],
        comment_list: vec![],
        saved_by_user: vec![],
    };
    let insert = col.insert_one(post, None).await;
    let inserted_id = insert.unwrap().inserted_id.as_i32().unwrap();
    let val = col.find_one(doc! { "_id":inserted_id }, None).await.unwrap();
    return PostDetail::from(val.unwrap());
}


pub async fn update_post(account: Account, update: UpdatePost) -> Result<PostDetail, ErrorMessage> {
    let col = connection_post().await;
    let date = Utc::now();
    let mut update_post = match col.find_one(doc! {"slug":update.slug}, None).await.unwrap() {
        None => { return Err(ErrorMessage::NotFound); }
        Some(x) => { x }
    };

    if !account.username.eq(&update_post.user_username) {
        return Err(ErrorMessage::NotOwned);
    }

    update_post.updated_at = date;
    if update.banner.as_ref().is_some() { update_post.banner = update.banner.unwrap(); }
    if update.title.as_ref().is_some() { update_post.title = update.title.unwrap(); }
    if update.content.as_ref().is_some() { update_post.content = update.content.unwrap(); }
    if update.tag.as_ref().is_some() { update_post.tag = update.tag.unwrap(); }
    if update.status.as_ref().is_some() { update_post.status = update.status.unwrap(); }

    match col.replace_one(doc! {"_id":update_post.id}, update_post.borrow(), None).await {
        Ok(_) => {}
        Err(err) => { std::panic::panic_any(err) }
    }
    return Ok(PostDetail::from(update_post));
}

pub async fn change_status(author: Account, slug: String) -> Result<Post, ErrorMessage> {
    let col = connection_post().await;
    let post = col.find_one(doc! {"slug":slug.to_owned(),"userUserName":author.username}, None).await.unwrap();
    if post.is_none() {
        return Err(ErrorMessage::NotFound);
    }
    println!("{:?}", post);
    match post.as_ref().unwrap().status {
        Status::Published => {
            match col.update_one(doc! {"slug":slug}, doc! {"$set":{"status":"Draft"}}, None).await {
                Ok(_) => {}
                Err(err) => { std::panic::panic_any(err) }
            }
        }
        Status::Draft => {
            match col.update_one(doc! {"slug":slug}, doc! {"$set":{"status":"Published"}}, None).await {
                Ok(_) => {}
                Err(err) => { std::panic::panic_any(err) }
            }
        }
    }
    match post {
        Some(p) => Ok(p),
        _ => Err(ErrorMessage::ServerError)
    }
}

pub async fn add_comment(comment: CreateComment, account: Account) -> Post {
    let col = connection_post().await;
    let mut post = col.find_one(doc! {"slug":&comment.slug}, None).await.unwrap().unwrap();
    let mut last_id = 1;
    let now = Utc::now();
    if !post.comment.is_empty() {
        for comment in post.comment.iter() {
            if comment.id >= last_id { last_id = comment.id };
        }
        last_id += 1;
    }
    let post_comment = Comment {
        id: last_id,
        content: comment.content,
        user_username: account.username,
        user_avatar: account.avatar,
        user_name: account.name,
        created_at: now,
        updated_at: now,
        interact: 0,
        parent_id: comment.parent_id,
        interact_list: vec![],
    };
    post.comment_count += 1;
    if !post.comment_list.contains(&account.id) {
        post.comment_list.insert(post.comment_list.len(), account.id);
    }
    post.comment.insert(post.comment.len(), post_comment);
    match col.replace_one(doc! {"slug":&comment.slug}, post, None).await {
        Ok(_) => {}
        Err(err) => { std::panic::panic_any(err) }
    }
    col.find_one(doc! {"slug":&comment.slug}, None).await.unwrap().unwrap()
}


pub async fn update_comment(comment: UpdateComment, account: Account) -> Result<PostDetail, ErrorMessage> {
    let col = connection_post().await;
    let post = col.find_one(doc! {"slug":&comment.slug_post}, None).await.unwrap();
    let now = Utc::now();
    let mut post = match post {
        None => { return Err(ErrorMessage::NotFound); }
        Some(p) => { p }
    };
    for (i, com) in post.comment.iter().enumerate() {
        if com.id == comment.id {
            if com.user_username != account.username { return Err(ErrorMessage::NotOwned); }
            post.comment[i].content = comment.content;
            post.comment[i].updated_at = now;
            break;
        }
    }
    match col.replace_one(doc! {"_id":&post.id}, &post, None).await {
        Ok(_) => {}
        Err(err) => { std::panic::panic_any(err) }
    }
    Ok(PostDetail::from(post))
}

pub async fn add_interact(slug: String, user_id: i32) -> Result<(), ErrorMessage> {
    let post_col = connection_post().await;
    let post_opt = post_col.find_one(doc! {"slug":slug.to_string()}, None).await.unwrap();
    match post_opt {
        None => {
            return Err(ErrorMessage::NotFound);
        }
        Some(post) => {
            if *&post.status == Status::Draft {
                return Err(ErrorMessage::BadRequest);
            }
            if post.reaction_list.contains(&user_id) {
                return Err(ErrorMessage::Duplicate);
            }
            let mut update_post = doc! {
        "$set":{
        "reactionCount":post.reaction_count+1
        }};
            if !post.reaction_list.contains(&user_id) {
                update_post.insert("$push", doc! {"reactionList":user_id});
            }
            let rs_post = post_col.update_one(doc! {"slug":slug.to_string()}, update_post, None).await;
            match rs_post {
                Ok(e) => {
                    println!("{:?}", e);
                }
                Err(err) => {
                    println!("{:?}", err);
                }
            }
            Ok(())
        }
    }
}

pub async fn remove_interact(slug: String, user_id: i32) -> Result<(), ErrorMessage> {
    let post_col = connection_post().await;
    let post = post_col.find_one(doc! {"slug":slug.to_string()}, None).await.unwrap().unwrap();
    return if post.reaction_list.contains(&user_id) {
        let user_col = connect_user().await;
        let mut update_post = doc! {
        "$set":{
        "reactionCount":post.reaction_count-1
        },
        "$pull":{
            "reactionList":user_id
        }
    };
        update_post.insert("$pull", doc! {"reactionList":user_id});
        match post_col.update_one(doc! {"slug":slug}, update_post, None).await {
            Ok(_) => {}
            Err(err) => { std::panic::panic_any(err) }
        }
        match user_col.update_one(doc! {"_id":user_id}, doc! {"$pull": {"reactionList":post.id}}, None).await {
            Ok(_) => {}
            Err(err) => { std::panic::panic_any(err) }
        }
        Ok(())
    } else {
        Err(ErrorMessage::NotFound)
    };
}

pub async fn search_post(account: &Option<Account>, keyword: String) -> Vec<Index> {
    let col = connection_post().await;
    let search_string = keyword;
    let query_find = doc! {
        "$and":[
            {"$or":[
            {"title":{"$regex":&search_string}},
            {"content":{"$regex":&search_string}},
        ]},{
            "status":"Published"
        }]};
    let cursor = col.find(query_find, None).await.unwrap();
    return map_index(cursor, account).await;
}

pub async fn search_post_by_username(account: &Option<Account>, username: &String) -> Vec<Index> {
    let col = connection_post().await;
    let search_string = username;
    let query_find = doc! {
        "$and":[{
            "userUserName":search_string
        }
            ,{
            "status":"Published"
        }]};
    let sort = FindOptions::builder().sort(doc! {
        "comment.createdAt":-1
    }).build();
    let cursor = col.find(query_find, sort).await.unwrap();
    return map_index(cursor, account).await;
}

pub async fn search_comment_post(account: &Option<Account>, keyword: String) -> Vec<Index> {
    let col = connection_post().await;
    let cursor = col.find(doc! {"comment.content":{"$regex":keyword}}, None).await.unwrap();
    return map_index(cursor, account).await;
}

// Use when username is already identify
pub async fn search_comment_content_by_username(account: &Option<Account>, username: &String) -> Vec<CommentInfoPage> {
    let col = connection_post().await;
    let mut result: Vec<CommentInfoPage> = vec![];
    let sort = FindOptions::builder().sort(doc! {
        "comment.createdAt":-1
    }).build();
    let mut cursor = col.find(doc! {"comment.username":username.to_owned()}, sort).await.unwrap();
    while let Some(post) = cursor.try_next().await.unwrap() {
        for comment in post.comment.to_owned() {
            if &comment.user_username == username {
                let mut parent: Option<CommentDetail> = None;
                if comment.parent_id == 0 {
                    parent = None;
                } else {
                    for com in &post.comment {
                        if com.id == comment.parent_id {
                            let mut temp = CommentDetail::from(com.to_owned());
                            match account {
                                None => {}
                                Some(acc) => {
                                    if com.interact_list.contains(&acc.id) { temp.interacted = true }
                                }
                            };
                            parent = Some(temp);
                        }
                    }
                };
                let mut child = CommentDetail::from(comment.to_owned());
                match account {
                    Some(acc) => {
                        if comment.interact_list.contains(&acc.id) { child.interacted = true }
                    }
                    None => {}
                }

                result.push(CommentInfoPage {
                    post_slug: post.slug.to_owned(),
                    post_title: post.title.to_owned(),
                    parent_comment: parent,
                    child_comment: child,
                })
            }
        }
    }
    result
}

pub async fn interact_comment(slug: String, id: i32, user_id: i32) -> Result<(), ErrorMessage> {
    let post_col = connection_post().await;
    let post = post_col.find_one(doc! {"slug":&slug}, None).await.unwrap().unwrap();
    let mut count = 0;

    for com in post.comment.iter() {
        if com.id == id {
            if com.interact_list.contains(&user_id) {
                return Err(ErrorMessage::Duplicate);
            }
            count = com.interact + 1;
        }
    }
    return match post_col.update_one(doc! {
        "slug":slug,
        "comment._id":id
    }, doc! {
        "$set":{
            "comment.$.interact":count
        },
        "$push":{
            "comment.$.interactList":user_id
        }
    }, None).await {
        Ok(_) => { Ok(()) }
        Err(_) => { Err(ErrorMessage::ServerError) }
    };
}

pub async fn un_interact_comment(slug: String, id: i32, user_id: i32) -> Result<(), ErrorMessage> {
    let post_col = connection_post().await;
    let post = post_col.find_one(doc! {"slug":&slug}, None).await.unwrap().unwrap();
    let mut count = 0;
    for com in post.comment.iter() {
        if com.id == id {
            if !com.interact_list.contains(&user_id) {
                return Err(ErrorMessage::BadRequest);
            }
            count = com.interact - 1;
            break;
        }
    }
    return match post_col.update_one(doc! {
        "slug":slug,
        "comment._id":id
    }, doc! {
        "$set":{
            "comment.$.interact":count
        },
        "$pull":{
            "comment.$.interactList":user_id
        }
    }, None).await {
        Ok(_) => { return Ok(()); }
        Err(_) => { Err(ErrorMessage::ServerError) }
    };
}

pub async fn index(user_id: Option<i32>, page: i32) -> Vec<Index> {
    let col = connection_post().await;
    let find_option = FindOptions::builder().sort(doc! {
            "createdAt":-1,
            "reactionCount":-1,
            "commentCount":-1
    }).skip({
        if page == 0 { 0 } else { ((page - 1) * 15) as u64 }
    }).limit(15).build();
    let cursor = col.find(doc! {"status":"Published"}, find_option).await.unwrap();
    let account = match user_id {
        None => { None }
        Some(x) => {
            let id = x;
            let user_col = connect_user().await;
            let usr = user_col.find_one(doc! { "_id": id}, None).await.unwrap();
            usr
        }
    };
    return map_index(cursor, &account).await;
}

pub async fn add_reading(account: Account, slug: String) -> Result<String, String> {
    let user_col = connect_user().await;
    let post_col = connection_post().await;
    let post = post_col.find_one(doc! {"slug":slug.to_owned()}, None).await.unwrap();
    match post {
        None => { return Err("Can not found post with provided slug".to_string()); }
        Some(p) => {
            if account.reading_list.contains(&p.id) {
                return Err("Already added".to_string());
            }
            match user_col.update_one(doc! { "_id":account.id}, doc! {"$push":{"readingList":p.id}}, None).await {
                Ok(_) => {}
                Err(_err) => { return Err("Can not add interact".to_string()); }
            }
        }
    }
    Ok(slug)
}

pub async fn remove_reading(account: Account, slug: String) -> Result<(), String> {
    let user_col = connect_user().await;
    let post_col = connection_post().await;
    let post = post_col.find_one(doc! {"slug":slug}, None).await.unwrap();
    match post {
        None => { return Err("Can not found post with provided slug".to_string()); }
        Some(p) => {
            if account.reading_list.contains(&p.id) {
                return Err("Already removed".to_string());
            }
            match user_col.update_one(doc! { "_id":account.id}, doc! {"$pull":{"readingList":p.id}}, None).await {
                Ok(_) => {}
                Err(_err) => { return Err("Can not remove interact".to_string()); }
            }
        }
    }
    Ok(())
}

pub async fn find_by_tag(user_id: Option<i32>, tag: String) -> Vec<Index> {
    let user = match user_id {
        None => { None }
        Some(x) => { get_user_by_id(x).await }
    };
    let col = connection_post().await;
    let cursor = col.find(doc! {"tag":tag}, None).await.unwrap();
    let result = map_index(cursor, &user).await;
    result
}

pub async fn get_post_dashboard(account: &Account) -> Vec<PostDetail> {
    let mut result: Vec<PostDetail> = vec![];
    let col = connection_post().await;
    let mut cursor = col.find(doc! {"userUserName":&account.username}, None).await.unwrap();
    while let Some(post) = cursor.try_next().await.unwrap() {
        let mut post_detail = PostDetail::from(post.to_owned());
        if post.reaction_list.contains(&account.id) {
            post_detail.interacted = true;
        }
        result.push(post_detail);
    }
    result
}

pub async fn get_tag_dashboard(account: &Account) -> Vec<TagList> {
    return tag::find_list_followed_tag(&account.followed_tag).await;
}

pub async fn get_post(user_id: Option<&i32>, slug: String) -> Result<PostDetail, ErrorMessage> {
    let col = connection_post().await;
    let cursor = col.find_one(doc! {"slug":slug}, None).await.unwrap();
    let post = match cursor {
        None => { return Err(ErrorMessage::NotFound); }
        Some(p) => { p }
    };
    match user_id {
        None => { Ok(PostDetail::from(post)) }
        Some(id) => {
            let interact_post = post.reaction_list.contains(id);
            let mut result = PostDetail::from(post.to_owned());
            result.interacted = interact_post;
            for com in post.comment.iter() {
                let mut comment = PostDetailComment::from(com.to_owned());
                if com.interact_list.contains(id) {
                    comment.interacted = true;
                    result.comment.push(comment)
                }
            }
            Ok(result)
        }
    }
}