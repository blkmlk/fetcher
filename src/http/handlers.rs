use std::io;
use actix_web::{HttpRequest, HttpResponse};

pub async fn get_entity(req: HttpRequest) -> HttpResponse {
    if let Some(id) = req.match_info().get("id") {
        println!("id: {}", id);
    }

    HttpResponse::Ok().finish()
}