use actix_web::web::Data;
use actix_web::{web, HttpResponse, Responder};
use chrono::Utc;
use sqlx::{PgPool};
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    pub name: String,
    pub email: String,
}

pub async fn subscribe(form_data: web::Form<FormData>, conn_pool: Data<PgPool>) -> impl Responder {
    let result = sqlx::query!(
        r#"
        insert into subscriptions(id, email, name, subscribed_at)
        values ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form_data.email,
        form_data.name,
        Utc::now()
    )
    .execute(conn_pool.as_ref())
    .await;
    match result {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            println!("Failed to insert subscription due to {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
