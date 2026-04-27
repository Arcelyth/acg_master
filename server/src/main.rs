use std::env;
use std::io::Result;

use actix_cors::Cors;
use actix_files::{Files, NamedFile};
use actix_session::{SessionMiddleware, config::PersistentSession, storage::RedisSessionStore};
use actix_web::cookie::Key;
use actix_web::web::ServiceConfig;
use actix_web::{App, HttpServer, http, middleware, web};
use time::Duration;

mod handler;
use handler::bangumi::*;

fn init_api(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope("/anime")
            .route("/start_game", web::post().to(start_new_game))
            .route("/verify_guess", web::post().to(verify_guess)),
    );
}

#[actix_web::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let host = env::var("HOST").expect("HOST is not set in .env file");
    let port = env::var("PORT").expect("PORT is not set in .env file");
    let server_url = format!("{host}:{port}");

    let redis_url = env::var("REDIS_URL").expect("REDIS_URL must be set");
    println!("redis_url: {}", redis_url);
    let redis_store = RedisSessionStore::new(redis_url)
        .await
        .expect("failed to create redis_store");

    let private_key = Key::from(
        &hex::decode(env::var("SESSION_SIGNING_KEY").expect("SESSION_SIGNING_KEY must be set"))
            .expect("failed to decode private_key"),
    );

    let app_env = env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());
    let is_prod = app_env == "production";

    let frontend_url =
        env::var("FRONTEND_URL").unwrap_or_else(|_| "http://127.0.0.1:8080".to_string());

    println!("Starting server at {server_url} in {app_env} mode");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin(&frontend_url)
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
            .allowed_headers(vec![
                "Content-Type",
                "Authorization",
                "Cookie",
                "Set-Cookie",
            ])
            .expose_headers(vec!["Set-Cookie"])
            .supports_credentials()
            .max_age(3600);

        App::new()
            .wrap(middleware::Logger::default())
            .wrap(
                SessionMiddleware::builder(redis_store.clone(), private_key.clone())
                    .cookie_secure(is_prod)
                    .session_lifecycle(PersistentSession::default().session_ttl(Duration::days(1)))
                    .build(),
            )
            .wrap(cors)
            .service(web::scope("/api/bangumi").configure(init_api))
            .service(Files::new("/", "/app/site").index_file("index.html"))
            .default_service(web::get().to(|| async {
        NamedFile::open_async("/app/site/index.html").await
    }))
    })
    .bind(&server_url)?
    .run()
    .await
}
