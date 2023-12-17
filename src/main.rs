#![allow(async_fn_in_trait)]

pub mod core;
pub mod handlers;
pub mod repositories;

use crate::core::service::Service;
use actix_web::{
    middleware::Logger,
    web::{delete, get, post, put, scope, Data},
    App, HttpServer,
};
use dotenv::dotenv;
use futures::io;
use handlers::{
    accept, assign_accepter, cancel_accepted_request, cancel_unaccepted_request, dismiss_accepter,
    finish_walk, record_walking_location, remove_acceptance, resign_acceptance, start_walk,
};
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
                scope("apis").service(
                    scope("walk_requests")
                        .route("", post().to(handlers::create_walk_request::<Mongodb>))
                        .route(
                            "nearby",
                            get().to(handlers::nearby_walk_requests::<Mongodb>),
                        )
                        .route("mine", get().to(handlers::my_walk_requests::<Mongodb>))
                        .route("/{id}/accepted_by", put().to(accept::<Mongodb>))
                        .route(
                            "/{id}/acceptances",
                            delete().to(remove_acceptance::<Mongodb>),
                        )
                        .route("/{id}/accepter/{uid}", put().to(assign_accepter::<Mongodb>))
                        .route(
                            "/{id}/accepter/{uid}",
                            delete().to(dismiss_accepter::<Mongodb>),
                        )
                        .route("/{id}/resign", delete().to(resign_acceptance::<Mongodb>))
                        .route(
                            "/{id}/accepted_by/{uid}",
                            delete().to(cancel_accepted_request::<Mongodb>),
                        )
                        .route("/{id}", delete().to(cancel_unaccepted_request::<Mongodb>))
                        .route("/{id}/start", put().to(start_walk::<Mongodb>))
                        .route("/{id}/finish", put().to(finish_walk::<Mongodb>))
                        .route(
                            "/{id}/locations",
                            post().to(record_walking_location::<Mongodb>),
                        ),
                ),
            )
    })
    .bind(config.listen_address)
    .expect("Can't bind to address")
    .run()
    .await
}
