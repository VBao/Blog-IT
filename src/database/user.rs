use std::borrow::Borrow;
use std::prelude::rust_2021::Option;

use argon2::Config;
use chrono::{Duration, Utc};
use futures::TryStreamExt;
use jsonwebtoken::{Algorithm, decode, DecodingKey, encode, EncodingKey, Header, Validation};
use mongodb::{bson::doc, Client, options::ClientOptions};
use mongodb::options::FindOneOptions;

use crate::constant;
use crate::constant::MONGODB_URL;
use crate::dto::user_dto::{AccountStore, CreateAccount, ShowAccountAdmin, SmallAccount};
use crate::error::ErrorMessage;
use crate::model::user::*;

const SALT: &str = "r5sAxyGpQ-vB";

pub async fn connect() -> mongodb::Collection<Account> {
    let mut conn = ClientOptions::parse(MONGODB_URL).await.unwrap();
    conn.app_name = Some("My App".to_string());
    let client = Client::with_options(conn).unwrap();
    let db = client.database("test");
    db.collection::<Account>("user")
}

pub async fn get_users() -> Vec<ShowAccountAdmin> {
    let mut rs: Vec<ShowAccountAdmin> = Vec::new();
    let col = connect().await;
    let mut cursor = col.find(None, None).await.unwrap();
    while let Some(account) = cursor.try_next().await.unwrap() {
        rs.push(ShowAccountAdmin::from(account));
    }
    return rs;
}

pub async fn get_user_by_id(id: i32) -> Option<Account> {
    let col = connect().await;
    let account = col.find_one(doc! {"_id":id}, None).await;
    match account {
        Ok(x) => { x }
        Err(x) => {
            println!("{:?}", x);
            Option::None
        }
    }
}

pub async fn log_in(usr: String, pwd: String) -> Result<AccountStore, &'static str> {
    let col = connect().await;
    let usr = col.find_one(doc! {"username":usr}, None).await.unwrap();
    match usr {
        Some(info) => {
            let check = argon2::verify_encoded(&info.password.as_ref(), pwd.as_ref()).unwrap();
            if check {
                let key = constant::SECRET_KEY.as_bytes();

                let claim = Claim {
                    sub: info.id.clone(),
                    exp: (Utc::now() + Duration::days(365)).timestamp() as usize,
                };
                let token = encode(
                    &Header::default(),
                    &claim,
                    &EncodingKey::from_secret(key),
                ).unwrap();
                col.update_one(doc! {"username":&info.username}, doc! {"$set":{"lastAccess":Utc::now()}}, None).await;
                let mut info = AccountStore::from(info);
                info.token = token;
                Ok(info)
            } else {
                return Err("Wrong user or password");
            }
        }
        None => return Err("Not found user")
    }
}

pub(crate) async fn get_info(username: String) -> Option<Account> {
    let col = connect().await;
    col.find_one(doc! {"username":username}, None).await.unwrap()
}

pub async fn sign_up(account: CreateAccount) -> Result<AccountStore, &'static str> {
    let col = connect().await;
    let id = {
        let sort = FindOneOptions::builder().sort(doc! {"_id":-1}).build();
        let last_user = col.find_one(None, sort).await.unwrap();
        if last_user.is_some() {
            last_user.unwrap().id + 1
        } else {
            1
        }
    };
    let password = {
        let config = Config::default();
        argon2::hash_encoded(account.password.as_ref(), SALT.as_ref(), config.borrow()).unwrap()
    };
    let now = Utc::now();
    let create_acc = Account {
        id,
        name: account.name,
        username: account.username.to_owned(),
        school_email: account.school_email,
        private_email: account.private_email,
        bio: if account.bio.is_some() { account.bio.unwrap() } else { "".to_string() },
        password,
        avatar: if account.avatar.is_some() { account.avatar.unwrap() } else { "".to_string() },
        admin: false,
        website: if account.website.is_some() { account.website.unwrap() } else { "".to_string() },
        last_access: now,
        created_at: now,
        updated_at: now,
        status: Status::Pending,
        followed_tag: if account.followed_tag.is_none() {
            vec![]
        } else {
            account.followed_tag.unwrap()
        },
        reading_list: vec![],
        followed_user: vec![],
    };
    match col.insert_one(create_acc, None).await {
        Ok(_) => {}
        Err(err) => { std::panic::panic_any(err) }
    }
    log_in(account.username, account.password).await
}

pub async fn create_admin(account: CreateAccount) {
    let col = connect().await;
    let id = {
        let sort = FindOneOptions::builder().sort(doc! {"_id":-1}).build();
        col.find_one(None, sort).await.unwrap().unwrap().id + 1
    };
    let password = {
        let config = Config::default();
        argon2::hash_encoded(account.password.as_ref(), SALT.as_ref(), config.borrow()).unwrap()
    };
    let now = Utc::now();
    let create_acc = Account {
        id,
        name: account.name,
        username: account.username,
        school_email: account.school_email,
        private_email: account.private_email,
        bio: "".to_string(),
        password,
        avatar: "".to_string(),
        admin: true,
        website: "".to_string(),
        last_access: now,
        created_at: now,
        updated_at: now,
        status: Status::Pending,
        followed_tag: vec![],
        reading_list: vec![],
        followed_user: vec![],
    };
    match col.insert_one(create_acc, None).await {
        Ok(_) => {}
        Err(err) => { std::panic::panic_any(err) }
    }
}

pub async fn find_by_id(id: i32) -> Result<Account, ()> {
    let usr = connect().await.find_one(doc! {"_id":id}, None).await;
    match usr {
        Ok(Some(x)) => Ok(x),
        _ => Err(())
    }
}


pub async fn add_reading(post_id: i32, user_id: i32) {
    let col = connect().await;
    match col.update_one(doc! {"_id":user_id}, doc! {"$push":doc! {"readingList":post_id}}, None).await {
        Ok(_) => {}
        Err(err) => { std::panic::panic_any(err) }
    }
}

pub async fn remove_reading(post_id: i32, user_id: i32) {
    let col = connect().await;
    match col.update_one(doc! {"_id":user_id}, doc! {"$pull":doc! {"readingList":post_id}}, None).await {
        Ok(_) => {}
        Err(err) => { std::panic::panic_any(err) }
    }
}

pub async fn search_by_username(keyword: String) -> Vec<Account> {
    let col = connect().await;
    let cursor = col.find(doc! {"username":{"$regex":keyword}}, None).await.unwrap();
    let rs = cursor.try_collect().await.unwrap_or_else(|_| vec![]);
    return rs;
}

pub async fn get_user_list_dashboard(user_list: &Vec<i32>) -> Vec<SmallAccount> {
    let mut rs: Vec<SmallAccount> = vec![];
    let col = connect().await;
    let query = doc! {
        "_id":{
            "$in":user_list
        }
    };
    let mut cursor = col.find(query, None).await.unwrap();
    while let Some(user) = cursor.try_next().await.unwrap() {
        let mut user = SmallAccount::from(user);
        user.followed = true;
        rs.push(user);
    }
    rs
}


pub async fn auth_get_id(token: &str) -> Result<i32, String> {
    let key = constant::SECRET_KEY.as_bytes();
    let decode = decode::<Claim>(
        token,
        &DecodingKey::from_secret(key),
        &Validation::new(Algorithm::HS256),
    );
    match decode {
        Ok(decoded) => {
            Ok(decoded.claims.sub)
        }
        Err(_) => {
            Err("Something wrong here".to_string())
        }
    }
}

pub async fn get_user_full(id: i32) -> Account {
    let col = connect().await;
    return col.find_one(doc! {"_id":id}, None).await.unwrap().unwrap();
}

pub async fn follow_user_toggle(user_id: i32, username_follow: String) -> Result<(), ErrorMessage> {
    let col = connect().await;
    let user_follow = col.find_one(doc! {"username":username_follow}, None).await.unwrap();
    return match user_follow {
        None => {
            Err(ErrorMessage::NotFound)
        }
        Some(user_follower) => {
            let follower = col.find_one(doc! {"_id":&user_id}, None).await.unwrap().unwrap();
            if follower.followed_user.contains(&user_follower.id) {
                col.update_one(doc! {"_id":user_id}, doc! {"$pull":{"followedUser":user_follower.id}}, None).await;
            } else {
                col.update_one(doc! {"_id":user_id}, doc! {"$push":{"followedUser":user_follower.id}}, None).await;
            }
            Ok(())
        }
    };
}