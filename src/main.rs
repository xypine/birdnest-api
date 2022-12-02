use anyhow::Result;
use futures::future;
use ndz::{get_drone_distance_to_ndz, NDZ_MIN_ALLOWED_DISTANCE};
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use reaktor::{drones::Drone, pilots::Pilot};
use serde::{Deserialize, Serialize};

mod ndz;
mod reaktor;

#[tokio::main]
async fn main() {
    let infringements = get_infringin_pilots().await.unwrap();
    println!("ID\t\tDISTANCE");
    infringements
        .iter()
        .for_each(|i| println!("{}\t{}", i.pilot.as_ref().unwrap().pilot_id, i.distance));
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
