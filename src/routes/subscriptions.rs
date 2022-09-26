use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse, ResponseError};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

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
) -> Result<HttpResponse, SubscribeError> {
    let new_subscriber = form_data.0.try_into()?;
    let subscription_id = insert_subscription(&new_subscriber, conn_pool.as_ref())
        .await
        .map_err(SubscribeError::InsertSubscriberError)?;
    let subscription_token = generate_token();
    insert_token(subscription_id, &subscription_token, conn_pool.as_ref()).await?;

    send_confirmation_email(
        new_subscriber,
        email_client.as_ref(),
        &base_url.as_ref().0,
        &subscription_token,
    )
    .await?;
    Ok(HttpResponse::Ok().finish())
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
) -> Result<(), StoreTokenError> {
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
        StoreTokenError(e)
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

pub struct StoreTokenError(sqlx::Error);

impl Display for StoreTokenError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Database error encountered when trying to store subscription token"
        )
    }
}

impl Debug for StoreTokenError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(&self.0, f)
    }
}

impl Error for StoreTokenError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.0)
    }
}

fn error_chain_fmt(e: &impl Error, f: &mut Formatter<'_>) -> std::fmt::Result {
    writeln!(f, "{}", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by \n\t{}", cause)?;
        current = cause.source();
    }
    Ok(())
}

pub enum SubscribeError {
    ValidationError(String),
    PoolError(sqlx::Error),
    InsertSubscriberError(sqlx::Error),
    TxCommitError(sqlx::Error),
    StoreTokenError(StoreTokenError),
    SendEmailError(reqwest::Error),
}

impl Display for SubscribeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SubscribeError::ValidationError(e) => write!(f, "{}", e),
            SubscribeError::PoolError(_) => write!(f, "Failed to create database pool"),
            SubscribeError::InsertSubscriberError(_) => write!(f, "Failed to insert subscriber"),
            SubscribeError::TxCommitError(_) => write!(f, "Failed to commit transaction"),
            SubscribeError::StoreTokenError(_) => write!(f, "Failed to store token"),
            SubscribeError::SendEmailError(_) => write!(f, "Failed to send email"),
        }
    }
}

impl Debug for SubscribeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl Error for SubscribeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            SubscribeError::ValidationError(_) => None,
            SubscribeError::StoreTokenError(e) => Some(e),
            SubscribeError::SendEmailError(e) => Some(e),
            SubscribeError::PoolError(e) => Some(e),
            SubscribeError::InsertSubscriberError(e) => Some(e),
            SubscribeError::TxCommitError(e) => Some(e),
        }
    }
}

impl ResponseError for SubscribeError {
    fn status_code(&self) -> StatusCode {
        match self {
            SubscribeError::ValidationError(_) => StatusCode::BAD_REQUEST,
            SubscribeError::PoolError(_)
            | SubscribeError::InsertSubscriberError(_)
            | SubscribeError::TxCommitError(_)
            | SubscribeError::StoreTokenError(_)
            | SubscribeError::SendEmailError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<StoreTokenError> for SubscribeError {
    fn from(e: StoreTokenError) -> Self {
        Self::StoreTokenError(e)
    }
}

impl From<reqwest::Error> for SubscribeError {
    fn from(e: reqwest::Error) -> Self {
        Self::SendEmailError(e)
    }
}

impl From<String> for SubscribeError {
    fn from(e: String) -> Self {
        Self::ValidationError(e)
    }
}
