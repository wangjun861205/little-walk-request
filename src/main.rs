#![allow(async_fn_in_trait)]

pub mod core;
pub mod handlers;
pub mod repositories;

use crate::core::service::Service;
use actix_web::{
    middleware::Logger,
    web::{get, post, scope, Data},
    App, HttpServer,
};
use dotenv::dotenv;
use futures::io;
use handlers::accept_request;
use mongodb::Client;
use nb_from_env::{FromEnv, FromEnvDerive};
use repositories::mongodb::Mongodb;

#[derive(FromEnvDerive)]
pub struct Config {
    pub listen_address: String,
    pub database_url: String,
    pub database_name: String,
    #[env_default("info")]
    pub log_level: String,
    #[env_default("%t %r %s %T")]
    pub log_format: String,
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    dotenv().ok();
    let config = Config::from_env();
    env_logger::init_from_env(env_logger::Env::default().default_filter_or(config.log_level));
    let db = Client::with_uri_str(&config.database_url)
        .await
        .expect("failed to connect to mongodb")
        .database(&config.database_name);
    let repository = Mongodb::new(db);
    let service = Service::new(repository);
    HttpServer::new(move || {
        let log_format = config.log_format.clone();
        App::new()
            .app_data(Data::new(service.clone()))
            .wrap(Logger::new(&log_format))
            .service(
                scope("walk_requests")
                    .route("", post().to(handlers::create_walk_request::<Mongodb>))
                    .route(
                        "nearby",
                        get().to(handlers::nearby_walk_requests::<Mongodb>),
                    )
                    .route("/{id}/acceptances", post().to(accept_request::<Mongodb>)),
            )
    })
    .bind(config.listen_address)
    .expect("Can't bind to address")
    .run()
    .await
}
