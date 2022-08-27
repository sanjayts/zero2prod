use actix_web::{HttpRequest, HttpResponse, Responder};

pub async fn health(_req: HttpRequest) -> impl Responder {
    HttpResponse::Ok().finish()
}
