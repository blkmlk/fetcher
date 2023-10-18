use std::error::Error;
use std::sync::Arc;
use actix_web::{FromRequest, Handler, HttpRequest, HttpResponse, Responder, web};
use actix_web::dev::ServiceRequest;
use actix_web::web::ServiceConfig;
use crate::http::server::State;
use crate::storage::connection::Row;

pub fn route_factory(cfg: &mut ServiceConfig) {
        cfg.route("/entity/{id}", web::get().to(handle));
}

async fn handle(req: HttpRequest) -> HttpResponse {
    if let Some(id) = req.match_info().get("id") {
        let data = req.app_data::<web::Data<State>>().unwrap();
        let mut eh = data.entity_handler.lock().unwrap();

        let res = eh.get_entity(id).await;

        let rows = match res {
            Ok(v) => v,
            Err(e) => return HttpResponse::InternalServerError().body(e.to_string())
        };

        for r in rows {
            for c in r.columns {
                println!("{} -> {}", c.0, c.1)
            }
        }
    }

    HttpResponse::Ok().finish()
}