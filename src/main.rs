use std::{thread, time::Duration};

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

#[tokio::main]
async fn main() {
    loop {
        record_infringements().await.unwrap();
        thread::sleep(Duration::from_secs(4));
    }
}

async fn record_infringements() -> Result<()> {
    let infringements = get_infringin_pilots().await?;
    let cache = INFRINGEMENTS.lock().await;
    for i in infringements {
        cache.insert(i.drone_serial_number.clone(), i).await;
    }
    println!("{} infringements", cache.entry_count());

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
}
