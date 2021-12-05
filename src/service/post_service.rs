use std::borrow::Borrow;
use actix_identity::Identity;
use actix_web::{HttpResponse, Responder, web};
use actix_web::web::{Json, Path};
use crate::database::post;
use mongodb::bson::doc;
use crate::database::post::find_by_tag;
use crate::database::user;
use crate::database::user::{get_user_by_id};
use crate::database::tag;
use crate::error::ErrorMessage;
use crate::dto::post_dto::*;
use crate::dto::user_dto::SmallAccount;
use crate::model::user::Account;
use crate::service::user_service::check_login;

pub async fn create_post(id: Identity, post: Json<CreatePost>) -> impl Responder {
    return match check_login(id) {
        Ok(user_id) => {
            let user = user::get_user_by_id(user_id).await.unwrap();
            let result = post::create_post(user, post.0).await;
            HttpResponse::Ok().json(doc! {"data":bson::to_bson(&result).unwrap()})
        }
        Err(err) => { err }
    };
}

pub async fn update_post(id: Identity, post: Json<UpdatePost>) -> impl Responder {
    match check_login(id) {
        Ok(user_id) => {
            let account = user::get_user_by_id(user_id).await.unwrap();
            match post::update_post(account, post.0).await {
                Ok(x) => {
                    HttpResponse::Ok().json(doc! {"data":bson::to_bson(&x).unwrap()})
                }
                Err(err) => { match_error(err) }
            }
        }
        Err(err) => { return err; }
    }
}


pub async fn change_status(id: Identity, slug: Path<String>) -> impl Responder {
    return match check_login(id) {
        Ok(user_id) => {
            match post::change_status(user::get_user_by_id(user_id).await.unwrap(), slug.0).await {
                Ok(_) => { HttpResponse::Ok().finish() }
                Err(_err) => { HttpResponse::InternalServerError().finish() }
            }
        }
        Err(err) => { err }
    };
}

pub async fn add_comment(id: Identity, comment: Json<CreateComment>) -> impl Responder {
    if id.identity().is_none() {
        return HttpResponse::Unauthorized().json(doc! {"error":"user not login"});
    }
    post::add_comment(comment.0, user::get_user_by_id(id.identity().unwrap().parse::<i32>().unwrap()).await.unwrap()).await;
    return HttpResponse::Ok().finish();
}

pub async fn update_comment(id: Identity, comment: Json<UpdateComment>) -> impl Responder {
    return match check_login(id) {
        Ok(user_id) => {
            let rs = post::update_comment(comment.0, user::get_user_by_id(user_id).await.unwrap()).await;
            match rs {
                Ok(x) => { HttpResponse::Ok().json(x) }
                Err(err) => {
                    match_error(err)
                }
            }
        }
        Err(err) => { return err; }
    };
}

pub async fn add_interact(id: Identity, slug_id: Path<String>) -> impl Responder {
    match id.identity() {
        None => { return HttpResponse::Unauthorized().json(doc! {"error":"user not login"}); }
        Some(user_id) => {
            match post::add_interact(slug_id.0, user_id.parse::<i32>().unwrap()).await {
                Err(err) => { match_error(err); }
                Ok(_) => { return HttpResponse::Ok().finish(); }
            }
        }
    }
    return HttpResponse::Ok().finish();
}

pub async fn remove_interact(id: Identity, slug_id: Path<String>) -> impl Responder {
    match check_login(id) {
        Ok(user_id) => {
            post::remove_interact(slug_id.0, user_id).await;
        }
        Err(err) => { return err; }
    }
    return HttpResponse::Ok().finish();
}

fn match_error(error: ErrorMessage) -> HttpResponse {
    return match error {
        ErrorMessage::NotOwned => { HttpResponse::Unauthorized().json(doc! {"error":"User not owned this comment"}) }
        ErrorMessage::NotFound => { HttpResponse::NotFound().json(doc! {"error":"Post not found"}) }
        ErrorMessage::Unauthorized => { HttpResponse::Unauthorized().json(doc! {"error":"User not owned this comment"}) }
        ErrorMessage::ServerError => { HttpResponse::InternalServerError().json(doc! {"error":"Please contact backend dev"}) }
        ErrorMessage::Duplicate => { HttpResponse::InternalServerError().json(doc! {"error":"Duplicate request please check again"}) }
    };
}

pub async fn search(id: Identity, keyword: Path<String>) -> impl Responder {
    let user: Option<Account> = match check_login(id) {
        Ok(id) => { get_user_by_id(id).await }
        Err(_) => { None }
    };
    let mut result = doc! {};
    let post_rs = post::search_post(user.borrow(), keyword.0.to_owned()).await;
    result.insert("post", bson::to_bson(&post_rs).unwrap());
    let comment_rs = post::search_comment_post(user.borrow(), keyword.0.to_owned()).await;
    result.insert("comment", bson::to_bson(&comment_rs).unwrap());
    let user_rs = match user
    {
        None => {
            user::search_by_username(keyword.0).await.into_iter().map(|p| { SmallAccount::from(p) }).collect()
        }
        Some(usr) => {
            let rs: Vec<SmallAccount> = user::search_by_username(keyword.0).await.into_iter().map(|p| {
                let mut rs = SmallAccount::from(p.to_owned());
                if usr.followed_user.contains(&p.id) {
                    rs.followed = true;
                }
                rs
            }).collect();
            rs
        }
    };
    result.insert("user", bson::to_bson(&user_rs).unwrap());
    return HttpResponse::Ok().json(doc! {"data":result});
}

pub async fn interact_comment(identity: Identity, web::Path((slug, id)): web::Path<(String, i32)>) -> impl Responder {
    match check_login(identity) {
        Ok(user_id) => {
            post::interact_comment(slug, id, user_id).await;
            return HttpResponse::Ok().finish();
        }
        Err(err) => { return err; }
    }
}

pub async fn un_interact_comment(identity: Identity, web::Path((slug, id)): web::Path<(String, i32)>) -> impl Responder {
    let user_id: i32 = identity.identity().unwrap().parse::<i32>().unwrap();
    post::un_interact_comment(slug, id, user_id).await;
    HttpResponse::Ok()
}

pub async fn index(identity: Identity, page: web::Path<i32>) -> impl Responder {
    let rs = post::index(identity.identity(), page.0).await;
    return HttpResponse::Ok().json(doc! {"data":bson::to_bson(&rs).unwrap()});
}

pub async fn add_reading(id: Identity, slug: web::Path<String>) -> impl Responder {
    match check_login(id) {
        Ok(user_id) => {
            match post::add_reading(get_user_by_id(user_id).await.unwrap(), slug.0).await {
                Ok(_) => { HttpResponse::Ok().finish() }
                Err(err) => { HttpResponse::InternalServerError().json(doc! {"error":err}) }
            }
        }
        Err(x) => { return x; }
    }
}

pub async fn remove_reading(id: Identity, slug: web::Path<String>) -> impl Responder {
    match check_login(id) {
        Ok(user_id) => {
            match post::remove_reading(get_user_by_id(user_id).await.unwrap(), slug.0).await {
                Ok(_) => { HttpResponse::Ok().finish() }
                Err(err) => { HttpResponse::InternalServerError().json(doc! {"error":err}) }
            }
        }
        Err(err) => { return err; }
    }
}

pub async fn get_tags(id: Identity) -> impl Responder {
    return match check_login(id) {
        Ok(id) => {
            let rs = tag::get_tags(Some(id)).await;
            HttpResponse::Ok().json(doc! {"data":bson::to_bson(&rs).unwrap()})
        }
        Err(_) => {
            let rs = tag::get_tags(None).await;
            HttpResponse::Ok().json(doc! {"data":bson::to_bson(&rs).unwrap()})
        }
    };
}

pub async fn get_tag(id: Identity, value: Path<String>) -> impl Responder {
    let user_id: Option<i32> = match check_login(id) {
        Ok(id) => { Some(id) }
        Err(_) => { None }
    };
    return match tag::get_tag(user_id.to_owned(), value.0.to_owned()).await {
        Ok(result) => {
            let mut resp = doc! {};
            let post = find_by_tag(user_id, value.0).await;
            resp.insert("tag", bson::to_bson(&result).unwrap());
            resp.insert("post", bson::to_bson(&post).unwrap());
            HttpResponse::Ok().json(doc! {"data": resp })
        }
        Err(err) => {
            match err {
                // ErrorMessage::NotOwned => {}
                ErrorMessage::NotFound => {
                    let msg = format!("Not found tag with value {}", value);
                    HttpResponse::NotFound().json(doc! {"error":msg})
                }
                // ErrorMessage::Unauthorized => {}
                ErrorMessage::ServerError => { HttpResponse::InternalServerError().json(doc! {"error":"Sorry we not validate this error"}) }
                // ErrorMessage::Duplicate => {}
                _ => { HttpResponse::InternalServerError().json(doc! {"error":"Sorry we not validate this error"}) }
            }
        }
    };
}

pub async fn get_post(id: Identity, slug: Path<String>) -> impl Responder {
    let user_id = match check_login(id) {
        Ok(x) => { Some(x) }
        Err(_) => { None }
    };
    return match post::get_post(user_id.as_ref(), slug.0).await {
        Ok(result) => {
            let resp = bson::to_bson(&result).unwrap();
            HttpResponse::Ok().json(doc! {"data":resp})
        }
        Err(err) => {
            match err {
                ErrorMessage::NotFound => { HttpResponse::NotFound().json(doc! {"error":"post not found"}) }
                _ => HttpResponse::InternalServerError().finish()
            }
        }
    };
}
