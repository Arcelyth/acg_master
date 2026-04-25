use std::env;
use std::io::Result;

use actix_cors::Cors;
use actix_session::{SessionMiddleware, config::PersistentSession, storage::RedisSessionStore};
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
            .route("/verify_guess", web::post().to(verify_guess)),
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
    let private_key = Key::from(
        &hex::decode(
            std::env::var("SESSION_SIGNING_KEY").expect("SESSION_SIGNING_KEY must be set"),
        )
        .expect("failed to create private_key"),
    );

    println!("Starting server at {server_url}");

    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allowed_origin("http://127.0.0.1:8080")
                    .allowed_origin("http://localhost:8080")
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
                    .allowed_headers(vec![
                        "Content-Type",
                        "Authorization",
                        "Cookie",
                        "Set-Cookie",
                    ])
                    .expose_headers(vec!["Set-Cookie"])
                    .supports_credentials()
                    .max_age(3600),
            )
            .wrap(
                SessionMiddleware::builder(redis_store.clone(), private_key.clone())
                    .cookie_secure(false)
                    .session_lifecycle(PersistentSession::default().session_ttl(Duration::days(1)))
                    .build(),
            )
            .wrap(middleware::Logger::default())
            .service(web::scope("/api/bangumi").configure(init_api))
    })
    .bind(&server_url)?
    .run()
    .await
}
