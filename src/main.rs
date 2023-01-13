mod cache;
mod config;
mod reaktor;
mod server;

use cache::INFRINGEMENTS;

use anyhow::Result;
use config::{get_drone_distance_to_ndz, NDZ_MIN_ALLOWED_DISTANCE};
use futures::future;
use log::{debug, error, info, log_enabled, Level};
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use reaktor::{drones::Drone, pilots::Pilot};
use serde::{Deserialize, Serialize};
use std::time::Duration;

// Tokio is used as the async runtime
#[tokio::main]
async fn main() {
    // Enable fancier logging
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();
    // Fetch infringements in the background
    let background_task = tokio::spawn(async {
        info!("Background task started!");
        loop {
            tokio::spawn(async {
                record_infringements()
                    .await
                    .expect("Failed to update infringements");
            });
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    });
    // Start the api
    server::start()
        .await
        .expect("Failed to start the api server");
    // Continues once the server has stopped
    info!("The server has stopped, stopping the background task...");
    background_task.abort();
    info!("Everything done, bye!")
}

/// Get infringements and save them to [INFRINGEMENTS]
async fn record_infringements() -> Result<()> {
    let infringements = get_infringin_pilots().await?;
    let cache = INFRINGEMENTS.lock().await;
    for i in infringements {
        let key = i.drone_serial_number.clone();
        match cache.get(&key) {
            Some(existing) => {
                let new = Infringement {
                    drone_serial_number: existing.drone_serial_number,
                    pilot: i.pilot,
                    updated_at: i.updated_at,
                    distance: existing.distance.min(i.distance),
                    x: if i.distance > existing.distance {
                        existing.x
                    } else {
                        i.x
                    },
                    y: if i.distance > existing.distance {
                        existing.y
                    } else {
                        i.y
                    },
                };
                cache.insert(key, new)
            }
            None => cache.insert(i.drone_serial_number.clone(), i),
        }
        .await;
    }
    debug!(
        "{} infringements in the last 10 minutes",
        cache.entry_count()
    );

    Ok(())
}

async fn get_infringin_pilots() -> Result<Vec<Infringement>> {
    let doc = reaktor::drones::get_drones().await?;
    let drones = doc.capture.drone;
    let tasks: Vec<_> = drones
        .par_iter()
        .map(|drone| DroneWithDistance {
            drone: drone.clone(),
            distance: get_drone_distance_to_ndz(drone),
        })
        .filter(|data| data.distance < NDZ_MIN_ALLOWED_DISTANCE)
        .map(|data| async move {
            let pilot = reaktor::pilots::get_pilot(&data.drone.serial_number)
                .await
                .ok();
            let drone_serial_number = data.drone.serial_number;
            Infringement {
                drone_serial_number,
                pilot,
                distance: data.distance,
                x: data.drone.position_x,
                y: data.drone.position_y,
                updated_at: chrono::offset::Utc::now().to_rfc3339(),
            }
        })
        .collect();

    Ok(future::join_all(tasks).await)
}

#[derive(Debug, Clone)]
pub struct DroneWithDistance {
    pub drone: Drone,
    pub distance: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize, paperclip::actix::Apiv2Schema)]
pub struct Infringement {
    pub drone_serial_number: String,
    pub pilot: Option<Pilot>,
    pub distance: f64,
    pub x: f64,
    pub y: f64,
    pub updated_at: String,
}
