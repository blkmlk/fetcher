use crate::http::handlers::get_entity;

pub fn route_factory(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.route("/entry/{id}", actix_web::web::get().to(get_entity));
}