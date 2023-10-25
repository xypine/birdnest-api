pub mod cache;
pub mod config;
pub mod reaktor;
pub mod server;
// optional features
pub mod features;

pub mod prelude {
    pub use crate::cache;
    pub use crate::config;
    pub use crate::get_infringements;
    pub use crate::reaktor;
    pub use crate::record_infringements;
    pub use crate::server;
}

use anyhow::Result;
use config::{get_drone_distance_to_ndz, NDZ_MIN_ALLOWED_DISTANCE};
use futures::future;
use log::debug;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use reaktor::{drones::Drone, pilots::Pilot};
use serde::{Deserialize, Serialize};

use cache::INFRINGEMENTS;

/// Get infringements and save them to [INFRINGEMENTS]
pub async fn record_infringements() -> Result<()> {
    let infringements = get_infringements().await?;
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

pub async fn get_infringements() -> Result<Vec<Infringement>> {
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
