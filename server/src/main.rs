use std::io::Result;
use std::env;

use actix_cors::Cors;
use actix_session::{SessionMiddleware, storage::RedisSessionStore, config::PersistentSession};
use actix_web::cookie::Key;
use actix_web::web::ServiceConfig;
use actix_web::{App, HttpServer, middleware, web};
use time::Duration;

mod handler;
use handler::*;

fn init_api(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope("/anime") 
            .route("/start_game", web::get().to(start_new_game))
            .route("/verify_guess", web::post().to(verify_guess))
    );
}

#[actix_web::main]
async fn main() -> Result<()> {
    // get env vars
    dotenvy::dotenv().ok();

    let host = env::var("HOST").expect("HOST is not set in .env file");
    let port = env::var("PORT").expect("PORT is not set in .env file");
    let server_url = format!("{host}:{port}");

    let redis_url = env::var("REDIS_URL").expect("REDIS_URL is not set in .env file");
    println!("redis_url: {}", redis_url);
    let redis_store = RedisSessionStore::new(redis_url)
        .await
        .expect("failed to create redis_store");
    let private_key = Key::from(&hex::decode(
        std::env::var("SESSION_SIGNING_KEY").expect("SESSION_SIGNING_KEY must be set"),
    ).expect("failed to create private_key"));

    println!("Starting server at {server_url}");
    HttpServer::new(move || {
        App::new()
            // Todo allowed_origin
            .wrap(Cors::permissive())
            .wrap(
                SessionMiddleware::builder(redis_store.clone(), private_key.clone())
                    .cookie_secure(false)
                    .session_lifecycle(
                        PersistentSession::default()
                            .session_ttl(Duration::days(3))
                    )
                    .build(),
            )
            .wrap(middleware::Logger::default())
            .service(web::scope("/api/bangumi").configure(init_api))
    })
    .bind(&server_url)?
    .run()
    .await
}


