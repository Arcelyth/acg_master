use std::io::Result;

use actix_cors::Cors;
use actix_web::web::ServiceConfig;
use actix_web::{App, HttpServer, middleware, web};
use std::env;

mod handler;
use handler::*;

fn init_api(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope("/anime") 
            .route("/get_random", web::get().to(get_random_anime))
    );
}

#[actix_web::main]
async fn main() -> Result<()> {
    // get env vars
    dotenvy::dotenv().ok();

    let host = env::var("HOST").expect("HOST is not set in .env file");
    let port = env::var("PORT").expect("PORT is not set in .env file");
    let server_url = format!("{host}:{port}");

    println!("Starting server at {server_url}");
    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .wrap(middleware::Logger::default())
            .service(web::scope("/api/bangumi").configure(init_api))
    })
    .bind(&server_url)?
    .run()
    .await
}
