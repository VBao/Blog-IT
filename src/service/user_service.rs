use actix_identity::Identity;
use actix_web::{web, Error, FromRequest, HttpResponse, Responder};
use actix_web::web::{Json, Path};
use futures::future::{err, ok, Ready};
use serde::{Deserialize};
use crate::database::user;
use crate::database::post;
use mongodb::bson::doc;
use argon2::{self, Config};
use crate::database::user::{get_user_by_id, get_user_list_dashboard};
use crate::dto::user_dto::{CreateAccount, UserPage};

#[derive(Deserialize)]
pub struct LoginInfo {
    username: String,
    password: String,
}

#[derive(Deserialize)]
pub struct Logged {
    id: String,
}

impl FromRequest for Logged {
    type Error = Error;
    type Future = Ready<Result<Logged, Error>>;
    type Config = ();

    fn from_request(
        req: &actix_web::HttpRequest,
        payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        if let Ok(identity) = Identity::from_request(req, payload).into_inner() {
            if let Some(some_json) = identity.identity() {
                if let Ok(user) = serde_json::from_str(&some_json) {
                    return ok(user);
                }
            }
        }
        err(HttpResponse::Unauthorized().into())
    }
}

pub async fn login(acc: web::Json<LoginInfo>, identity: Identity) -> impl Responder {
    // return HttpResponse::Ok().body(String::from(acc.pwd.to_owned()));
    let usr = acc.username.to_owned();
    let pwd = acc.password.to_owned();
    match user::log_in(usr, pwd).await {
        Ok(id) => {
            identity.remember(id.id.to_owned().to_string());
            let account = bson::to_bson(&id).unwrap();
            HttpResponse::Ok().json(doc! { "data": account })
        }
        _ => HttpResponse::Forbidden().json(doc! {"error":"username or password incorrect"})
    }
}


pub async fn test_password(password: Json<Logged>) -> impl Responder {
    let config = Config::default();
    let hash = argon2::hash_encoded(password.id.as_ref(), "r5sAxyGpQ-vB".as_ref(), &config).unwrap();
    HttpResponse::Ok().body(hash)
}

pub async fn users_get() -> impl Responder {
    return HttpResponse::Ok().json(doc! {"data":bson::to_bson(&user::get_users().await).unwrap()});
}

pub async fn get_user(identity: Identity, username: Path<String>) -> impl Responder {
    let user_id = match check_login(identity) {
        Ok(id) => { Some(id) }
        Err(_) => { None }
    };
    let account = match user_id {
        None => { None }
        Some(id) => {
            user::get_user_by_id(id).await
        }
    };
    let user_info = user::get_info(username.0.to_owned()).await;
    let recent_comment = post::search_comment_content_by_username(&account, &username).await;
    let recent_post = post::search_post_by_username(&account, &username).await;
    let count_tag = user_info.as_ref().unwrap().followed_tag.to_owned().iter().count() as i32;
    let count_post = recent_post.to_owned().iter().count() as i32;
    let count_comment = recent_comment.to_owned().iter().count() as i32;
    let mut usr = UserPage::from(user_info.clone().unwrap());
    if account.is_some() {
        let acc = account.unwrap();
        if acc.followed_user.contains(&user_info.unwrap().id) { usr.followed = true }
    }
    let result = doc! {
        "recentComment":bson::to_bson(&recent_comment).unwrap(),
        "recentPost":bson::to_bson(&recent_post).unwrap(),
        "userInfo":bson::to_bson(&usr).unwrap(),
        "summary":{
            "countTag":count_tag,
            "countComment":count_comment,
            "countPost":count_post
        }
    };
    return HttpResponse::Ok().json(doc! {"data":result});
}

pub async fn get_dashboard(id: Identity) -> impl Responder {
    return match check_login(id) {
        Ok(x) => {
            let mut response = doc! {};
            let user = get_user_by_id(x).await.unwrap();
            let post_list = post::get_post_dashboard(&user).await;
            response.insert("post", bson::to_bson(&post_list).unwrap());
            let tag_list = post::get_tag_dashboard(&user).await;
            response.insert("tag", bson::to_bson(&tag_list).unwrap());
            let user_list = get_user_list_dashboard(&user.followed_user).await;
            response.insert("following", bson::to_bson(&user_list).unwrap());
            HttpResponse::Ok().json(doc! { "data":response})
        }
        Err(err) => { err }
    };
}

pub async fn sign_up(acc: Json<CreateAccount>) -> impl Responder {
    match user::sign_up(acc.0).await {
        Ok(e) => HttpResponse::Created().json(e),
        Err(_) => HttpResponse::BadRequest().finish()
    }
}

pub fn check_login(id: Identity) -> Result<i32, HttpResponse> {
    match id.identity() {
        None => { Err(HttpResponse::Unauthorized().json(doc! {"error":"User not login"})) }
        Some(user_id) => {
            return Ok(user_id.parse::<i32>().unwrap());
        }
    }
}