use crate::{cache::INFRINGEMENTS, Infringement};
use actix_cors::Cors;
use actix_web::{get, middleware, App, HttpResponse, HttpServer, Result};

use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct InfringementResponse {
    pub infringements: Vec<Infringement>,
}

#[get("/infringements")]
async fn get_infridgements() -> Result<HttpResponse> {
    let cache = INFRINGEMENTS.lock().await;
    let infringements: Vec<_> = cache.iter().map(|i| i.1).collect();
    let out = InfringementResponse { infringements };
    return Ok(HttpResponse::Ok().json(out));
}

#[derive(Serialize, Debug)]
pub struct DronesResponse {
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    pub serials: Vec<String>,
}

#[get("/drones")]
async fn get_drones() -> Result<HttpResponse> {
    let drones = crate::cache::LATEST_DRONE_SNAPSHOT.lock().await;

    let mut x = vec![];
    let mut y = vec![];
    let mut serials = vec![];

    if let Some(drones) = &*drones {
        for drone in &drones.capture.drone {
            x.push(drone.position_x);
            y.push(drone.position_y);
            serials.push(drone.serial_number.clone());
        }
    }

    let out = DronesResponse { x, y, serials };
    return Ok(HttpResponse::Ok().json(out));
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
            .service(get_infridgements)
            .service(get_drones)
    })
    .bind(http_bind)?
    .run()
    .await
}
