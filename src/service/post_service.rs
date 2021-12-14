use std::borrow::Borrow;
use actix_web::{HttpRequest, HttpResponse, Responder, web};
use actix_web::web::{Json, Path};
use log::kv::Source;
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

pub async fn create_post(id: HttpRequest, post: Json<CreatePost>) -> impl Responder {
    return match check_login(id).await {
        Ok(user_id) => {
            let user = user::get_user_by_id(user_id).await.unwrap();
            if *&post.tag.len().to_owned() as i32 > 3 {
                return HttpResponse::BadRequest().json(doc! {"msg":"Tag list must not be more than 3 tag"});
            }
            let result = post::create_post(user, post.0).await;
            HttpResponse::Ok().json(doc! {"data":bson::to_bson(&result).unwrap()})
        }
        Err(err) => { err }
    };
}

pub async fn update_post(id: HttpRequest, post: Json<UpdatePost>) -> impl Responder {
    match check_login(id).await {
        Ok(user_id) => {
            if post.tag.is_some() {
                if *&post.tag.as_ref().unwrap().len() > 3 {
                    return HttpResponse::BadRequest().json(doc! {"msg":"Tag list must not be more than 3 tag"});
                }
            }
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


pub async fn change_status(id: HttpRequest, slug: Path<String>) -> impl Responder {
    return match check_login(id).await {
        Ok(user_id) => {
            match post::change_status(user::get_user_by_id(user_id).await.unwrap(), slug.0).await {
                Ok(_) => { HttpResponse::Ok().finish() }
                Err(_err) => { HttpResponse::InternalServerError().finish() }
            }
        }
        Err(err) => { err }
    };
}

pub async fn add_comment(id: HttpRequest, comment: Json<CreateComment>) -> impl Responder {
    match check_login(id).await {
        Ok(user_id) => {
            post::add_comment(comment.0, user::get_user_by_id(user_id).await.unwrap()).await;
            HttpResponse::Ok().finish()
        }
        Err(err) => { err }
    }
}

pub async fn update_comment(id: HttpRequest, comment: Json<UpdateComment>) -> impl Responder {
    return match check_login(id).await {
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

pub async fn add_interact(id: HttpRequest, slug_id: Path<String>) -> impl Responder {
    match check_login(id).await {
        Err(err) => { err }
        Ok(user_id) => {
            match post::add_interact(slug_id.0, user_id).await {
                Err(err) => {
                    match err {
                        ErrorMessage::NotFound => { HttpResponse::Ok().json(doc! {"msg":"Not found post with provided slug"}) }
                        ErrorMessage::Duplicate => { HttpResponse::Ok().json(doc! {"msg":"Already interacted"}) }
                        ErrorMessage::BadRequest => { HttpResponse::BadRequest().json(doc! {"msg":"Can not interact draft post"}) }
                        _ => { HttpResponse::InternalServerError().json(doc! {"msg":"Un-process exception"}) }
                    }
                }
                Ok(_) => { return HttpResponse::Ok().json(doc! {"msg":"Add-interact completed"}); }
            }
        }
    }
}

pub async fn remove_interact(id: HttpRequest, slug_id: Path<String>) -> impl Responder {
    return match check_login(id).await {
        Ok(user_id) => {
            match post::remove_interact(slug_id.0, user_id).await {
                Ok(_) => {
                    HttpResponse::Ok().json(doc! {"msg":"Un-interact completed"})
                }
                Err(err) => {
                    if err == ErrorMessage::NotFound {
                        return HttpResponse::BadRequest().json(doc! {"msg":"User not interact before"});
                    }
                    HttpResponse::InternalServerError().json(doc! {"msg":"Un-check exception"})
                }
            }
        }
        Err(err) => { err }
    };
}

fn match_error(error: ErrorMessage) -> HttpResponse {
    return match error {
        ErrorMessage::NotOwned => { HttpResponse::Unauthorized().json(doc! {"error":"User not owned this comment"}) }
        ErrorMessage::NotFound => { HttpResponse::NotFound().json(doc! {"error":"Post not found"}) }
        ErrorMessage::Unauthorized => { HttpResponse::Unauthorized().json(doc! {"error":"User not owned this comment"}) }
        ErrorMessage::ServerError => { HttpResponse::InternalServerError().json(doc! {"error":"Please contact backend dev"}) }
        ErrorMessage::Duplicate => { HttpResponse::InternalServerError().json(doc! {"error":"Duplicate request please check again"}) }
        ErrorMessage::BadRequest => { HttpResponse::BadRequest().json(doc! {"error":"Please re-check input"}) }
    };
}

pub async fn search(id: HttpRequest, keyword: Path<String>) -> impl Responder {
    let user: Option<Account> = match check_login(id).await {
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

pub async fn interact_comment(identity: HttpRequest, web::Path((slug, id)): web::Path<(String, i32)>) -> impl Responder {
    return match check_login(identity).await {
        Ok(user_id) => {
            post::interact_comment(slug, id, user_id).await;
            HttpResponse::Ok().finish()
        }
        Err(err) => { err }
    };
}

pub async fn un_interact_comment(req: HttpRequest, web::Path((slug, id)): web::Path<(String, i32)>) -> impl Responder {
    match check_login(req).await {
        Ok(user_id) => {
            post::un_interact_comment(slug, id, user_id).await;
            HttpResponse::Ok().finish()
        }
        Err(err) => { err }
    }
}

pub async fn index(req: HttpRequest, page: web::Path<i32>) -> impl Responder {
    return match check_login(req).await {
        Ok(user_id) => {
            let rs = post::index(Some(user_id), page.0).await;
            HttpResponse::Ok().json(doc! {"data":bson::to_bson(&rs).unwrap()})
        }
        Err(_) => {
            let rs = post::index(None, page.0).await;
            HttpResponse::Ok().json(doc! {"data":bson::to_bson(&rs).unwrap()})
        }
    };
}

pub async fn add_reading(id: HttpRequest, slug: web::Path<String>) -> impl Responder {
    match check_login(id).await {
        Ok(user_id) => {
            match post::add_reading(get_user_by_id(user_id).await.unwrap(), slug.0).await {
                Ok(_) => { HttpResponse::Ok().finish() }
                Err(err) => { HttpResponse::InternalServerError().json(doc! {"error":err}) }
            }
        }
        Err(x) => { return x; }
    }
}

pub async fn remove_reading(id: HttpRequest, slug: web::Path<String>) -> impl Responder {
    match check_login(id).await {
        Ok(user_id) => {
            match post::remove_reading(get_user_by_id(user_id).await.unwrap(), slug.0).await {
                Ok(_) => { HttpResponse::Ok().finish() }
                Err(err) => { HttpResponse::InternalServerError().json(doc! {"error":err}) }
            }
        }
        Err(err) => { return err; }
    }
}

pub async fn get_tags(id: HttpRequest) -> impl Responder {
    return match check_login(id).await {
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

pub async fn get_tag(id: HttpRequest, value: Path<String>) -> impl Responder {
    let user_id: Option<i32> = match check_login(id).await {
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

pub async fn get_post(id: HttpRequest, slug: Path<String>) -> impl Responder {
    let user_id = match check_login(id).await {
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
