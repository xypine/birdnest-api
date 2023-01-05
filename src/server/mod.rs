use crate::{cache::INFRINGEMENTS, Infringement};
use actix_cors::Cors;
use actix_web::{error, middleware, App, Error, HttpServer};
use actix_web_lab::web as web_lab;

use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};

use paperclip::actix::{
    api_v2_operation,
    web::{self, Json, Query},
    Apiv2Schema, OpenApiExt,
};

#[derive(Deserialize, Apiv2Schema)]
struct InfringementParams {
    /// Unix time stamp, optional
    /// Filters out infringements that have not been updated since min_updated_at
    min_updated_at: Option<i64>,
}
#[derive(Serialize, Debug, Apiv2Schema)]
pub struct InfringementResponse {
    pub infringements: Vec<Infringement>,
}

#[api_v2_operation]
/// Provides a list of recent infringements.
async fn get_infringements(
    params: Query<InfringementParams>,
) -> Result<Json<InfringementResponse>, Error> {
    let cache = INFRINGEMENTS.lock().await;
    let mut infringements: Vec<_> = cache.iter().map(|i| i.1).collect();
    if let Some(min_updated_at_timestamp) = &params.min_updated_at {
        let min_updated_at = DateTime::<Utc>::from_utc(
            NaiveDateTime::from_timestamp_millis(*min_updated_at_timestamp)
                .ok_or(error::ErrorBadRequest("invalid timestamp"))?,
            Utc,
        );
        infringements = infringements
            .iter()
            .filter(move |i| {
                return DateTime::parse_from_rfc3339(&i.updated_at).unwrap() > min_updated_at;
            })
            .map(|i| i.clone()) // bad
            .collect();
    }
    let out = InfringementResponse { infringements };
    return Ok(Json(out));
}

#[derive(Serialize, Debug, Apiv2Schema)]
pub struct DronesResponse {
    /// A list of drone x-coordinates
    pub x: Vec<f64>,
    /// A list of drone y-coordinates
    pub y: Vec<f64>,
    /// A list of drone serials
    pub serials: Vec<String>,
}

#[api_v2_operation]
/// Lists properties of drones currently within the sensors range.
/// The n:th index in every array belongs to the same drone
async fn get_drones() -> Result<Json<DronesResponse>, Error> {
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
    return Ok(Json(out));
}

pub async fn start() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=debug");
    env_logger::init();

    let http_bind = std::env::var("HTTP_BIND").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    println!("Starting server on http://{}:...", http_bind);

    HttpServer::new(move || {
        App::new()
            // Enable CORS
            .wrap(Cors::permissive())
            // Enable logger
            .wrap(middleware::Logger::default())
            .service(web_lab::Redirect::new("/", "/swagger"))
            .wrap_api()
            .service(web::resource("/infringements").route(web::get().to(get_infringements)))
            .service(web::resource("/drones").route(web::get().to(get_drones)))
            .with_json_spec_at("/openapi.json")
            .with_swagger_ui_at("/swagger")
            .build()
    })
    .bind(http_bind)?
    .run()
    .await
}
