use actix_web::dev::Server;
use actix_web::middleware::Logger;
use actix_web::web::Data;
use actix_web::{web, App, HttpServer};
use sqlx::PgPool;
use std::net::TcpListener;

use crate::routes::*;

pub fn run(listener: TcpListener, conn_pool: PgPool) -> std::io::Result<Server> {
    let conn_pool = Data::new(conn_pool);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .route("/subscriptions", web::post().to(subscribe))
            .route("/health_check", web::get().to(health))
            .app_data(conn_pool.clone())
    })
    .listen(listener)?
    .run();
    Ok(server)
}
