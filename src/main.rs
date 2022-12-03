use std::time::Duration;

use anyhow::Result;
use futures::future;
use ndz::{get_drone_distance_to_ndz, NDZ_MIN_ALLOWED_DISTANCE};
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use reaktor::{drones::Drone, pilots::Pilot};
use serde::{Deserialize, Serialize};

mod cache;
use cache::INFRINGEMENTS;
mod ndz;
mod reaktor;

mod server;

#[tokio::main]
async fn main() {
    let _background_task = tokio::spawn(async {
        println!("Background task started!");
        loop {
            tokio::spawn(async {
                record_infringements().await.unwrap();
            });
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    });
    server::start().await.unwrap();
    // Runs when the server has stopped
    println!("Bye!");
}

async fn record_infringements() -> Result<()> {
    let infringements = get_infringin_pilots().await?;
    let cache = INFRINGEMENTS.lock().await;
    for i in infringements {
        let key = i.drone_serial_number.clone();
        match cache.get(&key) {
            Some(existing) => cache.insert(
                key,
                Infringement {
                    drone_serial_number: existing.drone_serial_number,
                    pilot: i.pilot,
                    distance: existing.distance.min(i.distance),
                    x: i.x,
                    y: i.y,
                    updated_at: i.updated_at,
                },
            ),
            None => cache.insert(i.drone_serial_number.clone(), i),
        }
        .await;
    }
    //println!("{} infringements", cache.entry_count());

    Ok(())
}

async fn get_infringin_pilots() -> Result<Vec<Infringement>> {
    let doc = reaktor::drones::get_drones().await?;
    let drones = doc.capture.drone;
    let tasks: Vec<_> = drones
        .par_iter()
        .map(|drone| DroneDistance {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DroneDistance {
    pub drone: Drone,
    pub distance: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Infringement {
    pub drone_serial_number: String,
    pub pilot: Option<Pilot>,
    pub distance: f64,
    pub x: f64,
    pub y: f64,
    pub updated_at: String,
}
