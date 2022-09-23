use actix_web::{web, HttpResponse, Responder};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

use sqlx::PgPool;
use time::OffsetDateTime;
use tracing;

use crate::domain::NewSubscriber;
use crate::email_client::EmailClient;
use crate::startup::ApplicationBaseUrl;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    pub name: String,
    pub email: String,
}

fn generate_token() -> String {
    let mut rng = thread_rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form_data, conn_pool, email_client, base_url),
    fields(
        subscriber_name=%form_data.name,
        subscriber_email=%form_data.email
    )
)]
pub async fn subscribe(
    form_data: web::Form<FormData>,
    conn_pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    base_url: web::Data<ApplicationBaseUrl>,
) -> impl Responder {
    let new_subscriber = match form_data.0.try_into() {
        Ok(ns) => ns,
        Err(_e) => return HttpResponse::BadRequest().finish(),
    };
    let subscription_id = match insert_subscription(&new_subscriber, conn_pool.as_ref()).await {
        Ok(subscriber_id) => subscriber_id,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let subscription_token = generate_token();
    if insert_token(subscription_id, &subscription_token, conn_pool.as_ref())
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError().finish();
    }

    if send_confirmation_email(
        new_subscriber,
        email_client.as_ref(),
        &base_url.as_ref().0,
        &subscription_token,
    )
    .await
    .is_err()
    {
        return HttpResponse::InternalServerError().finish();
    }
    HttpResponse::Ok().finish()
}

#[tracing::instrument(
    name = "Sending confirmation email to users",
    skip(new_subscriber, email_client)
)]
async fn send_confirmation_email(
    new_subscriber: NewSubscriber,
    email_client: &EmailClient,
    base_url: &str,
    token: &str,
) -> Result<(), reqwest::Error> {
    let confirmation_link = format!(
        "{}/subscriptions/confirm?subscription_token={}",
        base_url, token
    );
    email_client
        .send_email(
            new_subscriber.email,
            "Welcome!",
            &format!(
                "Welcome to our newsletter. Click <a href=\"{}\">here</a>",
                confirmation_link
            ),
            &format!(
                "Welcome to our newsletter. Click here -- {}",
                confirmation_link
            ),
        )
        .await
}

#[tracing::instrument(name = "Stored subscription token in the database", skip(pool))]
async fn insert_token(
    subscription_id: Uuid,
    subscription_token: &str,
    pool: &PgPool,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT into subscription_tokens(subscription_token, subscription_id)
        values ($1, $2)
        "#,
        subscription_token,
        subscription_id,
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to save subscription token -- error is {:?}", e);
        e
    })?;
    Ok(())
}

#[tracing::instrument(
    name = "Saving subscriber details in database",
    skip(new_subscriber, pool)
)]
async fn insert_subscription(
    new_subscriber: &NewSubscriber,
    pool: &PgPool,
) -> Result<Uuid, sqlx::Error> {
    let subscriber_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        insert into subscriptions(id, email, name, subscribed_at, status)
        values ($1, $2, $3, $4, 'pending_confirmation')
        "#,
        subscriber_id,
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
    Ok(subscriber_id)
}
