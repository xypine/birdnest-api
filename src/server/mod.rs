use crate::{cache::INFRINGEMENTS, Infringement};
use actix_cors::Cors;
use actix_web::{get, http::Error, middleware, App, HttpResponse, HttpServer};
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct Greet {
    pub infringements: Vec<Infringement>,
}

#[get("/")]
async fn greet() -> Result<HttpResponse, Error> {
    let cache = INFRINGEMENTS.lock().await;
    let infringements: Vec<_> = cache.iter().map(|i| i.1).collect();
    let out = Greet { infringements };
    return Ok::<HttpResponse, Error>(HttpResponse::Ok().json(out));
}

pub async fn start() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=debug");
    env_logger::init();

    let http_bind = std::env::var("HTTP_BIND").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    println!("Starting server on {}:...", http_bind);

    HttpServer::new(move || {
        App::new()
            // Enable CORS
            .wrap(Cors::permissive())
            // Enable logger
            .wrap(middleware::Logger::default())
            .service(greet)
    })
    .bind(http_bind)?
    .run()
    .await
}
