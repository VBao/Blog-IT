use actix_web::{HttpRequest, HttpResponse, Responder, web};
use actix_web::web::{Json, Path};
use argon2::{self, Config};
use mongodb::bson::doc;
use serde::Deserialize;

use crate::database::post;
use crate::database::user;
use crate::database::user::{get_user_by_id, get_user_full, get_user_list_dashboard};
use crate::dto::user_dto::{CreateAccount, UserPage};
use crate::error::ErrorMessage;

#[derive(Deserialize)]
pub struct LoginInfo {
    username: String,
    password: String,
}

#[derive(Deserialize)]
pub struct Logged {
    id: String,
}

pub async fn login(acc: web::Json<LoginInfo>) -> impl Responder {
    let usr = acc.username.to_owned();
    let pwd = acc.password.to_owned();
    match user::log_in(usr, pwd).await {
        Ok(id) => {
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

pub async fn users_get(req: HttpRequest) -> impl Responder {
    match check_login(req).await {
        Ok(id) => {
            return if check_admin(id).await {
                HttpResponse::Ok().json(doc! {"data":bson::to_bson(&user::get_users().await).unwrap()})
            } else {
                HttpResponse::Unauthorized().json(doc! {"msg":"only admin can access"})
            };
        }
        Err(err) => { err }
    }
}


pub async fn get_user(req: HttpRequest, username: Path<String>) -> impl Responder {
    let user_id = match check_login(req).await {
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

pub async fn get_dashboard(req: HttpRequest) -> impl Responder {
    return match check_login(req).await {
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
        Ok(e) => HttpResponse::Created().json(doc! {"data":bson::to_bson(&e).unwrap()}),
        Err(_) => HttpResponse::BadRequest().finish()
    }
}


pub async fn check_login(req: HttpRequest) -> Result<i32, HttpResponse> {
    match req.headers().get("Authorization") {
        Some(auth) => {
            let val: Vec<&str> = auth.to_str().unwrap().split("Bearer").collect();
            let token = val[1].trim();
            let val = user::auth_get_id(token).await;
            return match val {
                Ok(id) => {
                    Ok(id)
                }
                Err(err) => {
                    Err(HttpResponse::Unauthorized().body(err))
                }
            };
        }
        None => {
            Err(HttpResponse::Unauthorized().json(doc! {"msg":"User not login"}))
        }
    }
}

pub(crate) async fn check_admin(id: i32) -> bool {
    get_user_full(id).await.admin
}

pub async fn create_list(list: Json<Vec<CreateAccount>>) -> impl Responder {
    for acc in list.0 {
        user::sign_up(acc).await;
    }
    HttpResponse::Ok().json(doc! {"data":bson::to_bson(&user::get_users().await).unwrap()})
}

pub async fn follow_user_toggle(req: HttpRequest, username_following: Path<String>) -> impl Responder {
    return match check_login(req.to_owned()).await {
        Ok(id) => {
            match user::follow_user_toggle(id.to_owned(), username_following.0).await {
                Ok(_) => {
                    let mut response = doc! {};
                    let user = get_user_by_id(id).await.unwrap();
                    let post_list = post::get_post_dashboard(&user).await;
                    response.insert("post", bson::to_bson(&post_list).unwrap());
                    let tag_list = post::get_tag_dashboard(&user).await;
                    response.insert("tag", bson::to_bson(&tag_list).unwrap());
                    let user_list = get_user_list_dashboard(&user.followed_user).await;
                    response.insert("following", bson::to_bson(&user_list).unwrap());
                    HttpResponse::Ok().json(doc! { "data":response})
                }
                Err(err) => {
                    match err {
                        ErrorMessage::NotFound => {
                            HttpResponse::NotFound().json(doc! {"msg":"not found user with provided name"})
                        }
                        _ => {
                            HttpResponse::InternalServerError().json(doc! {"msg":"un-check exception"})
                        }
                    }
                }
            }
        }
        Err(err) => {
            err
        }
    };
}