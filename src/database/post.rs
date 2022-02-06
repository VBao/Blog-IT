use std::borrow::Borrow;
use std::option::Option::Some;
use std::prelude::rust_2021::Option;

use chrono::{DateTime, Utc};
use futures::TryStreamExt;
use mongodb::{bson::doc, Client, Collection, Cursor, options::ClientOptions};
use mongodb::options::{FindOneOptions, FindOptions};
use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;
use slug::slugify;

use crate::constant::MONGODB_URL;
use crate::database::tag;
use crate::database::user::{connect as connect_user, get_user_by_id};
use crate::dto::post_dto::{CommentDetail, CommentInfoPage, CreateComment, CreatePost, Index, MorePost, PostDetail, PostDetailComment, PostDetailPage, ShortPost, ShortPostAdmin, UpdateComment, UpdatePost};
use crate::dto::tag_dto::TagList;
use crate::dto::user_dto::PostDetailUser;
use crate::error::ErrorMessage;
use crate::model::post::*;
use crate::model::tag::Tag;
use crate::model::user::Account;

pub async fn connection_post() -> Collection<Post> {
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
            banner: if post.banner.is_none() { "".to_string() } else { post.banner.unwrap() },
            title: post.title,
            created_at: post.created_at,
            updated_at: post.updated_at,
            status: Status::Published,
            tag_list: post.tag,
            // saved_count:post.saved_by_user.len() as i32,
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
    let mut conn = ClientOptions::parse(MONGODB_URL).await.unwrap();
    conn.app_name = Some("My App".to_string());
    let client = Client::with_options(conn).unwrap();
    let db = client.database("test");
    db.collection::<Tag>("tag")
}

pub async fn create_post(account: Account, create: CreatePost) -> PostDetail {
    let col = connection_post().await;
    let tag_col = connection_tag().await;
    let now: DateTime<Utc> = Utc::now();
    let sort = FindOneOptions::builder().sort(doc! {"_id":-1}).build();
    let last_post = col.borrow().find_one(None, sort).await.unwrap();
    let slug = slug_generate(&create.title, &account.username).await;
    let post = Post {
        id: if last_post.is_some() { last_post.unwrap().id + 1 } else { 1 },
        user_username: account.username,
        user_avatar: account.avatar,
        user_name: account.name,
        slug,
        banner: create.banner,
        title: create.title.to_string(),
        content: create.content.to_string(),
        created_at: now.into(),
        updated_at: now.into(),
        status: create.status,
        tag: create.tag,
        comment: vec![],
        comment_count: 0,
        reaction_count: 0,
        reaction_list: vec![],
        comment_list: vec![],
        saved_by_user: vec![],
    };
    let insert = col.insert_one(post, None).await;
    let inserted_id = insert.unwrap().inserted_id.as_i32().unwrap();
    let val = col.find_one(doc! { "_id":inserted_id }, None).await.unwrap().unwrap();

    for tag_val in &val.tag {
        let tag = tag_col.find_one(doc! {"value":tag_val}, None).await.unwrap().unwrap();
        match tag_col.update_one(doc! {"_id":tag.id}, doc! {"$set":{"post":tag.post+1}}, None).await {
            Ok(_) => {}
            Err(err) => { std::panic::panic_any(err) }
        }
    }
    return PostDetail::from(val);
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
    if update.banner.as_ref().is_some() { update_post.banner = update.banner; }
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

pub async fn add_comment(comment: CreateComment, account: Account) -> Result<PostDetail, String> {
    let col = connection_post().await;
    let mut post = match col.find_one(doc! {"slug":&comment.slug}, None).await.unwrap() {
        None => { return Err("Post not found".to_string()); }
        Some(p) => { p }
    };
    let mut last_id = 1;
    let now = Utc::now();
    if !post.comment.is_empty() {
        let mut check = false;
        if comment.parent_id != 0 {
            for comment_iter in post.comment.iter() {
                if &comment_iter.id == &comment.parent_id {
                    check = true;
                    break;
                }
            }
        } else {
            check = true;
        }
        if check {
            for comment_iter in post.comment.iter() {
                if comment_iter.id >= last_id { last_id = comment_iter.id };
            }
            last_id += 1;
        } else {
            return Err("Not found parent-comment with provided id".to_string());
        }
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
    Ok(PostDetail::from(col.find_one(doc! {"slug":&comment.slug}, None).await.unwrap().unwrap()))
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

pub async fn add_interact(slug: String, user_id: i32) -> Result<PostDetailPage, ErrorMessage> {
    let post_col = connection_post().await;
    let post_opt = post_col.find_one(doc! {"slug":slug.to_string()}, None).await.unwrap();
    return match post_opt {
        None => {
            Err(ErrorMessage::NotFound)
        }
        Some(post) => {
            if *&post.status == Status::Draft {
                return Err(ErrorMessage::BadRequest);
            }
            if post.reaction_list.contains(&user_id) {
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
                match post_col.update_one(doc! {"slug":slug.to_owned()}, update_post, None).await {
                    Ok(_) => {}
                    Err(err) => { std::panic::panic_any(err) }
                }
                match user_col.update_one(doc! {"_id":user_id}, doc! {"$pull": {"reactionList":post.id}}, None).await {
                    Ok(_) => {}
                    Err(err) => { std::panic::panic_any(err) }
                }
                return match get_post(Some(&user_id), slug).await {
                    Ok(post_rs) => { Ok(post_rs) }
                    Err(_) => { Err(ErrorMessage::ServerError) }
                };
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

            match get_post(Some(&user_id), slug).await {
                Ok(post_rs) => { Ok(post_rs) }
                Err(_) => { Err(ErrorMessage::ServerError) }
            }
        }
    };
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
    let mut cursor = col.find(doc! {"comment.userUserName":username.to_owned()}, sort).await.unwrap();
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

pub async fn interact_comment(slug: String, id: i32, user_id: i32) -> Result<PostDetailPage, ErrorMessage> {
    let post_col = connection_post().await;
    let post = post_col.find_one(doc! {"slug":&slug}, None).await.unwrap().unwrap();
    let mut count = 0;

    for com in post.comment.iter() {
        if com.id == id {
            if com.interact_list.contains(&user_id) {
                count = com.interact - 1;
                return match post_col.update_one(doc! {
        "slug":slug.to_owned(),
        "comment._id":id
    }, doc! {
        "$set":{
            "comment.$.interact":count
        },
        "$pull":{
            "comment.$.interactList":user_id
        }
    }, None).await {
                    Ok(_) => {
                        return match get_post(Some(&user_id), slug).await {
                            Ok(post_rs) => { Ok(post_rs) }
                            Err(_) => { Err(ErrorMessage::ServerError) }
                        };
                    }
                    Err(_) => { Err(ErrorMessage::ServerError) }
                };
            }
            count = com.interact + 1;
        }
    }
    return match post_col.update_one(doc! {
        "slug":slug.to_owned(),
        "comment._id":id
    }, doc! {
        "$set":{
            "comment.$.interact":count
        },
        "$push":{
            "comment.$.interactList":user_id
        }
    }, None).await {
        Ok(_) => {
            return match get_post(Some(&user_id), slug).await {
                Ok(post_rs) => { Ok(post_rs) }
                Err(_) => { Err(ErrorMessage::ServerError) }
            };
        }
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

pub async fn reading_process(account: Account, slug: String) -> Result<PostDetailPage, String> {
    let user_col = connect_user().await;
    let post_col = connection_post().await;
    let post = post_col.find_one(doc! {"slug":slug.to_owned()}, None).await.unwrap();
    return match post {
        None => { return Err("Can not found post with provided slug".to_string()); }
        Some(p) => {
            if account.reading_list.contains(&p.id) {
                match post_col.update_one(doc! {"slug":slug.to_owned()}, doc! {"$pull":{"savedByUser":account.id}}, None).await {
                    Ok(_) => {}
                    Err(_) => { return Err("Internal Server Error".to_string()); }
                }
                match user_col.update_one(doc! { "_id":account.id}, doc! {"$pull":{"readingList":p.id}}, None).await {
                    Ok(_) => {
                        return match get_post(Some(&account.id), slug).await {
                            Ok(post_rs) => { Ok(post_rs) }
                            Err(_) => { Err("Internal Server Error".to_string()) }
                        };
                    }
                    Err(_err) => { Err("Can not remove interact".to_string()) }
                }
            } else {
                match post_col.update_one(doc! { "slug":slug.to_owned()}, doc! {"$push":{"savedByUser":account.id}}, None).await {
                    Ok(_) => {}
                    Err(_) => { return Err("can not update".to_string()); }
                }
                return match user_col.update_one(doc! { "_id":account.id}, doc! {"$push":{"readingList":p.id}}, None).await {
                    Ok(_) => {
                        match get_post(Some(&account.id), slug).await {
                            Ok(post_rs) => { Ok(post_rs) }
                            Err(_) => { return Err("Internal Server Error".to_string()); }
                        }
                    }
                    Err(_err) => { Err("Can not add interact".to_string()) }
                };
            }
        }
    };
    // Ok(slug)
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

pub async fn get_post_dashboard(account: &Account) -> Vec<ShortPost> {
    let mut result: Vec<ShortPost> = vec![];
    let col = connection_post().await;
    let mut cursor = col.find(doc! {"userUserName":&account.username}, None).await.unwrap();
    while let Some(post) = cursor.try_next().await.unwrap() {
        let post_detail = ShortPost::from(post.to_owned());
        // let mut post_detail = PostDetail::from(post.to_owned());
        // if post.reaction_list.contains(&account.id) {
        //     post_detail.interacted = true;
        // }
        result.push(post_detail);
    }
    result
}

pub async fn get_tag_dashboard(account: &Account) -> Vec<TagList> {
    return tag::find_list_followed_tag(&account.followed_tag).await;
}

pub async fn get_post(user_id: Option<&i32>, slug: String) -> Result<PostDetailPage, ErrorMessage> {
    let col = connection_post().await;
    let user_col = connect_user().await;

    let cursor = col.find_one(doc! {"slug":slug.to_owned()}, None).await.unwrap();
    let post = match cursor {
        None => { return Err(ErrorMessage::NotFound); }
        Some(p) => { p }
    };
    let posted_user = user_col.find_one(doc! {"username":&post.user_username}, None).await.unwrap().unwrap();

//  Post Detail
    let (post_detail, post_user) = match user_id {
        None => { (PostDetail::from(post), PostDetailUser::from(posted_user)) }
        Some(usr_id) => {
            let mut post_detail_result = PostDetail::from(post.to_owned());
            post_detail_result.comment = Vec::new();
            post_detail_result.interacted = post.reaction_list.contains(usr_id);
            post_detail_result.saved = post.saved_by_user.contains(usr_id);
            for com in post.comment.iter() {
                let mut comment = PostDetailComment::from(com.to_owned());
                if com.interact_list.contains(usr_id) {
                    comment.interacted = true;
                }
                post_detail_result.comment.push(comment);
            }
            // Check followed user
            let reader_user = user_col.find_one(doc! {"_id":user_id}, None).await.unwrap().unwrap();
            let post_user = PostDetailUser {
                username: posted_user.username,
                name: posted_user.name,
                avatar: posted_user.avatar,
                followed: reader_user.followed_user.contains(&posted_user.id),
                bio: posted_user.bio,
            };

            (post_detail_result, post_user)
        }
    };

//  More Post
    let more_post = {
        let mut result: Vec<MorePost> = Vec::new();
        let find_option = FindOptions::builder().sort(doc! {
            "createdAt":-1,
            "reactionCount":-1,
            "commentCount":-1
        }).limit(5).build();
        let mut cursor = col.find(doc! {"$and":[
            {"status":"Published"},
            {"userUserName":&post_detail.user_username},
        ]}, find_option).await.unwrap();
        while let Some(post_detail) = cursor.try_next().await.unwrap() {
            result.push(MorePost::from(post_detail))
        }
        result
    };

    Ok(PostDetailPage{
        more_post,
        user_info: post_user,
        post_detail
    })
}

pub async fn toggle_save_post(user_id: i32, slug: String) -> Result<PostDetail, ErrorMessage> {
    let post_col = connection_post().await;
    let user_col = connect_user().await;
    let post = match post_col.find_one(doc! {"slug":&slug}, None).await.unwrap() {
        None => { return Err(ErrorMessage::NotFound); }
        Some(post) => { post }
    };

    if post.saved_by_user.contains(&user_id) {
        match user_col.update_one(doc! {"_id":user_id}, doc! {"$pull":{"readingList":post.id}}, None).await {
            Ok(_) => {}
            Err(_) => { return Err(ErrorMessage::ServerError); }
        }
        match post_col.update_one(doc! {"_id":slug}, doc! {"$pull":{"savedByUser":user_id}}, None).await {
            Ok(_) => {}
            Err(_) => { return Err(ErrorMessage::ServerError); }
        }
    } else {
        match post_col.update_one(doc! {"slug":slug}, doc! {"$push":{"savedByUser":user_id}}, None).await {
            Ok(_) => {}
            Err(_) => { return Err(ErrorMessage::ServerError); }
        }
        match user_col.update_one(doc! {"_id":user_id}, doc! {"$push":{"readingList":post.id}}, None).await {
            Ok(_) => {}
            Err(_) => { return Err(ErrorMessage::ServerError); }
        }
    }
    let result_post = post_col.find_one(doc! {"_id":post.id}, None).await.unwrap().unwrap();
    return Ok(PostDetail::from(result_post));
}

pub async fn toggle_follow_tag(user_id: i32, tag_val: String) -> Result<bool, ErrorMessage> {
    let tag_col = connection_tag().await;
    let user_col = connect_user().await;
    let tag = match tag_col.find_one(doc! {"value":tag_val}, None).await.unwrap() {
        None => { return Err(ErrorMessage::NotFound); }
        Some(value) => { value }
    };

    return match user_col.find_one(doc! {"$and":[
        {"_id":&user_id},
        {"followedTag":{"$in":[&tag.id]}}
    ]}, None).await.unwrap() {
        None => {
            match user_col.update_one(doc! {"_id":&user_id}, doc! {"$push":{"followedTag":&tag.id}}, None).await {
                Ok(_) => {
                    Ok(true)
                }
                Err(_) => { Err(ErrorMessage::ServerError) }
            }
        }
        Some(_) => {
            match user_col.update_one(doc! {"_id":&user_id}, doc! {"$pull":{"followedTag":&tag.id}}, None).await {
                Ok(_) => {
                    Ok(false)
                }
                Err(_) => { Err(ErrorMessage::ServerError) }
            }
        }
    };
}

pub async fn posts() -> Vec<ShortPostAdmin> {
    let find_option = FindOptions::builder().sort(doc! {
            "createdAt":-1,
            "reactionCount":-1,
            "commentCount":-1
    }).build();
    let post_col = connection_post().await;
    let mut cursor = post_col.find(doc! {"status":"Published"}, find_option).await.unwrap();
    let mut res: Vec<ShortPostAdmin> = Vec::new();
    while let Some(post) = cursor.try_next().await.unwrap() {
        res.push(ShortPostAdmin::from(post));
    }
    return res;
}

pub async fn delete_post(user_id: &i32, slug: String) -> Result<(), ErrorMessage> {
    let col = connection_post().await;
    let user_col = connect_user().await;
    let tag_col = connection_tag().await;

    let user = user_col.find_one(doc! {"_id":user_id}, None).await.unwrap().unwrap();
    let post_opt = col.find_one(doc! {"slug":&slug}, None).await.unwrap();
    return match post_opt {
        None => { Err(ErrorMessage::NotFound) }
        Some(post) => {
            if post.user_username != user.username {
                return Err(ErrorMessage::Unauthorized);
            }

            for tag_val in &post.tag {
                let tag = tag_col.find_one(doc! {"value":tag_val}, None).await.unwrap().unwrap();
                match tag_col.update_one(doc! {"_id":tag.id}, doc! {"$set":{"post":tag.post-1}}, None).await {
                    Ok(_) => {}
                    Err(err) => { std::panic::panic_any(err) }
                }
            }

            match col.delete_one(doc! {"slug":&slug}, None).await {
                Ok(_) => {}
                Err(_) => { return Err(ErrorMessage::ServerError); }
            }

            let user_col = connect_user().await;
            match user_col.update_many(doc! {"readingList":&post.id}, doc! {"$pull":{"readingList":&post.id}}, None).await {
                Ok(_) => { Ok(()) }
                Err(_) => { Err(ErrorMessage::ServerError) }
            }
        }
    };
}