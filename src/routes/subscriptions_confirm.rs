use actix_web::{web, HttpResponse, Responder};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(name = "Confirm pending subscription", skip(conn_pool, parameters))]
pub async fn confirm(
    conn_pool: web::Data<PgPool>,
    parameters: web::Query<Parameters>,
) -> impl Responder {
    let mut transaction = match conn_pool.begin().await {
        Ok(transaction) => transaction,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };
    let subscription_id =
        match get_subscriber_id(parameters.subscription_token.as_ref(), &mut transaction).await {
            Err(_err) => return HttpResponse::InternalServerError().finish(),
            Ok(None) => return HttpResponse::Unauthorized().finish(),
            Ok(Some(id)) => id,
        };
    if confirm_subscriber(subscription_id, &mut transaction)
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError().finish();
    }
    if transaction.commit().await.is_err() {
        return HttpResponse::InternalServerError().finish();
    }
    HttpResponse::Ok().finish()
}

#[tracing::instrument(
    name = "Retrieve subscriber ID from subscription token",
    skip(transaction, subscription_id)
)]
async fn confirm_subscriber<'a>(
    subscription_id: Uuid,
    transaction: &mut Transaction<'a, Postgres>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
    UPDATE subscriptions SET status = 'confirmed' WHERE id = $1
    "#,
        subscription_id
    )
    .execute(transaction)
    .await
    .map_err(|e| {
        tracing::error!("Failed to persist confirmed status -- error is {:?}", e);
        e
    })?;
    Ok(())
}

#[tracing::instrument(
    name = "Retrieve subscriber ID from subscription token",
    skip(transaction, subscription_token)
)]
async fn get_subscriber_id<'a>(
    subscription_token: &str,
    transaction: &mut Transaction<'a, Postgres>,
) -> Result<Option<Uuid>, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT subscription_id FROM subscription_tokens WHERE subscription_token = $1
    "#,
        subscription_token
    )
    .fetch_optional(transaction)
    .await
    .map_err(|e| {
        tracing::error!("Failed to retrieve subscription -- error is {:?}", e);
        e
    })?;
    Ok(row.map(|r| r.subscription_id))
}
