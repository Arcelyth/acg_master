use actix_web::{HttpResponse, Responder};

pub async fn get_random_anime() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "data": "omg",
        "page": 10,
        "total_pages": 10
    }))
}
