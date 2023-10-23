use actix_web::{FromRequest, Handler, HttpRequest, HttpResponse, Responder, web};
use actix_web::web::ServiceConfig;
use serde_json::json;
use crate::domain::fetcher::Value;
use crate::http::server::State;

pub fn route_factory(cfg: &mut ServiceConfig) {
        cfg.route("/id/{id}", web::get().to(handle));
}

async fn handle(req: HttpRequest) -> HttpResponse {
    if let Some(id) = req.match_info().get("id") {
        let data = req.app_data::<web::Data<State>>().unwrap();
        let mut eh = data.entity_handler.lock().unwrap();

        let resp = match eh.get_entity(id).await {
            Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
            Ok(v) => v
        };

        let mut obj = serde_json::Map::new();
        for (k, v) in resp {
            let mut in_obj = serde_json::Map::new();
            for (k, v) in v {
                let val = match v {
                    Value::String(vv) => serde_json::Value::String(vv),
                    Value::Array(vv) => serde_json::Value::Array(vv.iter().map(|e| {
                        serde_json::Value::String(e.to_string())
                    }).collect())
                };
                in_obj.insert(k, val);
            }
            obj.insert(k, serde_json::Value::Object(in_obj));
        }

        return HttpResponse::Ok().json(obj);
    }

    HttpResponse::Ok().finish()
}