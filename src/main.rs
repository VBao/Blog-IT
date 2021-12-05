mod service;
pub mod database;

use actix_cors::Cors;
use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_web::{web, App, HttpServer, };
use actix_web::middleware::Logger;
use service::*;

mod model;
mod error;
mod dto;
mod config;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG","info");
    std::env::set_var("RUST_BACKTRACE","1");
    env_logger::init();
    HttpServer::new(|| {
        let logger=Logger::default();
        let cors = Cors::default()
            .allow_any_header()
            .allow_any_method()
            .allow_any_origin()
            .max_age(3600);
        App::new()
            .wrap(logger)
            .wrap(cors)
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(&[0; 32])
                    .name("authorization")
                    .secure(false),
            ))
            .route("/login", web::post().to(user_service::login))
            .route("/signup", web::post().to(user_service::sign_up))
            .route("/index/{page}", web::get().to(post_service::index))
            .route("/argon", web::post().to(user_service::test_password))
            .route("/search/{keyword}", web::get().to(post_service::search))
            .service(
                web::scope("/post")
                    .route("/detail/{slug}", web::get().to(post_service::get_post))
                    .route("/tags", web::get().to(post_service::get_tags))
                    .route("/tag/{value}", web::get().to(post_service::get_tag)).route("/create", web::post().to(post_service::create_post))
                    .route("/update", web::post().to(post_service::update_post))
                    .route("/change-status/{slug}", web::get().to(post_service::change_status))
                    .route("/comment", web::post().to(post_service::add_comment))
                    .route("/edit-comment", web::post().to(post_service::update_comment))
                    .route("/interact/{slug_id}", web::get().to(post_service::add_interact))
                    .route("/un-interact/{slug_id}", web::get().to(post_service::remove_interact))
                    .route("/add-reading/{slug_id}", web::get().to(post_service::add_reading))
                    .route("/remove-reading/{slug_id}", web::get().to(post_service::remove_reading))
                    .route("/interact-comment/{slug}/{id}", web::get().to(post_service::interact_comment))
                    .route("/un-interact-comment/{slug}/{id}", web::get().to(post_service::un_interact_comment))
            ).service(
            web::scope("/admin")
        ).service(
            web::scope("/user")
                .route("/info/{username}", web::get().to(user_service::get_user))
                .route("/dashboard", web::get().to(user_service::get_dashboard))
                .service(
                    web::scope("admin")
                        .route("/users", web::get().to(user_service::users_get))
                )
        )
    })
        .bind("0.0.0.0:8040")?
        .run()
        .await
}

