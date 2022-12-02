use anyhow::Result;
use futures::future;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use reaktor::{drones::Drone, pilots::Pilot};

mod reaktor;

#[tokio::main]
async fn main() {
    let pilots_to_report = get_infringin_pilots().await.unwrap();
    println!("{:?}", pilots_to_report);
}

const NDZ_CENTER_X: f64 = 250000.0;
const NDZ_CENTER_Y: f64 = 250000.0;
const NDZ_MIN_ALLOWED_DISTANCE: f64 = 250000.0;
fn is_drone_in_ndz(drone: &Drone) -> bool {
    let distance = f64::sqrt(
        (drone.position_x - NDZ_CENTER_X).powi(2) + (drone.position_y - NDZ_CENTER_Y).powi(2),
    );
    return distance < NDZ_MIN_ALLOWED_DISTANCE;
}

async fn get_infringin_pilots() -> Result<Vec<Pilot>> {
    let doc = reaktor::drones::get_drones().await.unwrap();
    let drones = doc.capture.drone;
    let tasks: Vec<_> = drones
        .par_iter()
        .filter(|drone| is_drone_in_ndz(drone))
        .map(|drone| reaktor::pilots::get_pilot(&drone.serial_number))
        .collect();

    future::join_all(tasks).await.into_iter().collect()
}
