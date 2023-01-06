use crate::{cache::INFRINGEMENTS, Infringement};
use actix_cors::Cors;
use actix_web::{error, middleware, App, Error, HttpServer};
use actix_web_lab::web as web_lab;

use chrono::DateTime;
use serde::{Deserialize, Serialize};

use paperclip::actix::{
    api_v2_operation,
    web::{self, Json, Query},
    Apiv2Schema, OpenApiExt,
};

#[derive(Deserialize, Apiv2Schema)]
struct InfringementParams {
    /// An optional RFC3339 time stamp,
    /// filters out infringements that have not been updated since min_updated_at.
    /// In javascript you can use date.toISOString();
    #[openapi(example = "2023-01-06T13:45:40.503Z")]
    min_updated_at: Option<String>,
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
        let min_updated_at = DateTime::parse_from_rfc3339(min_updated_at_timestamp)
            .map_err(|e| error::ErrorBadRequest(e))?;
        infringements = infringements
            .iter()
            .filter(move |i| {
                return DateTime::parse_from_rfc3339(&i.updated_at)
                    .expect("Failed to parse server-created time string")
                    > min_updated_at;
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

use paperclip::v2::models::DefaultApiRaw;
use paperclip::v2::models::Info;
pub async fn start() -> std::io::Result<()> {
    // Enable logging
    std::env::set_var("RUST_LOG", "actix_web=debug");
    env_logger::init();

    let http_bind = std::env::var("HTTP_BIND").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    println!("Starting server on http://{}:...", http_bind);

    HttpServer::new(move || {
        let mut spec = DefaultApiRaw::default();
        // Insert package metadata
        spec.info = Info {
            version: env!("CARGO_PKG_VERSION").to_string(),
            title: env!("CARGO_PKG_NAME").to_string(),
            description: Some(env!("CARGO_PKG_DESCRIPTION").to_string()),
            ..Default::default()
        };
        // Build the api
        App::new()
            // Enable CORS
            .wrap(Cors::permissive())
            // Enable logger
            .wrap(middleware::Logger::default())
            // Redirect / to /swagger
            .service(web_lab::Redirect::new("/", "/swagger"))
            // Init routes with openapi
            .wrap_api_with_spec(spec)
            .service(web::resource("/infringements").route(web::get().to(get_infringements)))
            .service(web::resource("/drones").route(web::get().to(get_drones)))
            // Schema routes
            .with_json_spec_at("swagger.json")
            .with_json_spec_v3_at("openapi.json")
            .with_swagger_ui_at("/swagger")
            .with_rapidoc_at("/rapidoc")
            .build()
    })
    .bind(http_bind)?
    .run()
    .await
}
