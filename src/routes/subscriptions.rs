use actix_web::{web, HttpResponse, Responder};

#[derive(serde::Deserialize)]
pub struct FormData {
    pub name: String,
    pub email: String,
}

pub async fn subscribe(_form_data: web::Form<FormData>) -> impl Responder {
    HttpResponse::Ok().finish()
}
