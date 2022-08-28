use actix_web::{web, HttpResponse, Responder};
use sqlx::PgPool;
use time::OffsetDateTime;
use tracing::instrument::Instrument;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    pub name: String,
    pub email: String,
}

pub async fn subscribe(
    form_data: web::Form<FormData>,
    conn_pool: web::Data<PgPool>,
) -> impl Responder {
    let req_id = Uuid::new_v4();
    let span = tracing::info_span!(
        "Adding a new subscriber",
        request_id = %req_id,
        name = %form_data.name,
        email = %form_data.email
    );
    // VERY IMPORTANT: Ensure that we don't use `let _` as opposed to `let _guard`.
    // The former will simply  load up the guard and drop it immediately whereas the later
    // will enter the span and keep it open till the end of subscribe function which is what we want!
    let _span_guard = span.enter();

    let query_span = tracing::info_span!(
        "Try saving subscriber details in database",
        %req_id
    );
    let result = sqlx::query!(
        r#"
        insert into subscriptions(id, email, name, subscribed_at)
        values ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form_data.email,
        form_data.name,
        OffsetDateTime::now_utc(),
    )
    .execute(conn_pool.as_ref())
    .instrument(query_span)
    .await;
    match result {
        Ok(_) => {
            tracing::info!(
                "request_id {} - Finished saving subscription details in a database",
                req_id
            );
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            tracing::error!("Failed to save subscription -- error is {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
