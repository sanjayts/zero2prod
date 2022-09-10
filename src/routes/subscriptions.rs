use actix_web::{web, HttpResponse, Responder};

use sqlx::PgPool;
use time::OffsetDateTime;
use tracing;

use crate::domain::NewSubscriber;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    pub name: String,
    pub email: String,
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form_data, conn_pool),
    fields(
        subscriber_name=%form_data.name,
        subscriber_email=%form_data.email
    )
)]
pub async fn subscribe(
    form_data: web::Form<FormData>,
    conn_pool: web::Data<PgPool>,
) -> impl Responder {
    let new_subscriber = match form_data.0.try_into() {
        Ok(ns) => ns,
        Err(_e) => return HttpResponse::BadRequest().finish(),
    };
    match insert_subscription(&new_subscriber, conn_pool.as_ref()).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[tracing::instrument(
    name = "Saving subscriber details in database",
    skip(new_subscriber, pool)
)]
async fn insert_subscription(
    new_subscriber: &NewSubscriber,
    pool: &PgPool,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        insert into subscriptions(id, email, name, subscribed_at)
        values ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        OffsetDateTime::now_utc(),
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to save subscription -- error is {:?}", e);
        e
    })?;
    Ok(())
}
