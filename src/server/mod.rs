use crate::{cache::INFRINGEMENTS, Infringement};
use actix_cors::Cors;
use actix_web::{error, get, middleware, web::Query, App, HttpResponse, HttpServer, Result};

use chrono::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct InfringementParams {
    min_updated_at: Option<String>,
}
#[derive(Serialize, Debug)]
pub struct InfringementResponse {
    pub infringements: Vec<Infringement>,
}

#[get("/infringements")]
async fn get_infridgements(params: Query<InfringementParams>) -> Result<HttpResponse> {
    let cache = INFRINGEMENTS.lock().await;
    let mut infringements: Vec<_> = cache.iter().map(|i| i.1).collect();
    if let Some(min_updated_at_str) = &params.min_updated_at {
        let min_updated_at = DateTime::parse_from_rfc2822(&min_updated_at_str)
            .map_err(|e| error::ErrorBadRequest(e.to_string()))?;
        infringements = infringements
            .iter()
            .filter(move |i| {
                return DateTime::parse_from_rfc2822(&i.updated_at).unwrap() > min_updated_at;
            })
            .map(|i| i.clone()) // bad
            .collect();
    }
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
