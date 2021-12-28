use actix_web::{HttpRequest, HttpResponse, Responder};
use actix_web::web::Json;
use mongodb::bson::doc;

use crate::database::tag;
use crate::database::tag::update;
use crate::dto::tag_dto::{CreateTag, UpdateTag};
use crate::error::ErrorMessage;
use crate::user_service::{check_admin, check_login};

pub async fn get_tags_admin(req: HttpRequest) -> impl Responder {
    match check_login(req).await {
        Ok(id) => {
            if check_admin(id).await {
                HttpResponse::Ok().json(doc! {"data":bson::to_bson(&tag::tags().await).unwrap()})
            } else {
                HttpResponse::Unauthorized().json(doc! {"msg":"only admin can access"})
            }
        }
        Err(err) => { err }
    }
}

pub async fn create_tag(req: HttpRequest, tag_create: Json<CreateTag>) -> impl Responder {
    match check_login(req).await {
        Ok(id) => {
            if check_admin(id).await {
                match tag::create_tag(tag_create.0).await {
                    Ok(tag) => {
                        HttpResponse::Ok().json(doc! {"data":bson::to_bson(&tag).unwrap()})
                    }
                    Err(err) => {
                        match err {
                            ErrorMessage::Duplicate => { HttpResponse::Unauthorized().json(doc! {"msg":"only admin can access"}) }
                            _ => { HttpResponse::InternalServerError().json(doc! {"msg":"Un-process exception"}) }
                        }
                    }
                }
            } else {
                HttpResponse::Unauthorized().json(doc! {"msg":"only admin can access"})
            }
        }
        Err(err) => { err }
    }
}

pub async fn update_tag(req: HttpRequest, tag_update: Json<UpdateTag>) -> impl Responder {
    match check_login(req).await {
        Ok(id) => {
            if check_admin(id).await {
                match update(tag_update.0).await {
                    Ok(tags) => {
                        let mut res = doc! {};
                        res.insert("msg", "success");
                        res.insert("data", bson::to_bson(&tags).unwrap());
                        HttpResponse::Ok().json(res)
                    }
                    Err(err) => {
                        match err {
                            ErrorMessage::NotFound => { return HttpResponse::NotFound().json(doc! {"msg":"not found tag with provided id"}); }
                            ErrorMessage::Duplicate => { return HttpResponse::BadRequest().json(doc! {"msg":"already have tag with provided value"}); }
                            _ => { HttpResponse::InternalServerError().json(doc! {"msg":"uncheck exception"}) }
                        }
                    }
                }
            } else {
                return HttpResponse::Unauthorized().json(doc! {"msg":"only admin can access"});
            }
        }
        Err(err) => { err }
    }
}

pub async fn create_list(list: Json<Vec<CreateTag>>) -> impl Responder {
    for tag in list.0 {
        tag::create_tag(tag).await;
    }
    HttpResponse::Ok().json(doc! {"data":bson::to_bson(&tag::tags().await).unwrap()})
}