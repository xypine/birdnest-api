use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use super::PILOTS_ENDPOINT;
use crate::cache::PILOT_CACHE;
use crate::features::replay::{get_replay_status, load_replay_pilots, ReplayStatus};

use log::info;

pub async fn get_pilot(drone_serial_number: &String) -> Result<Pilot> {
    let key = drone_serial_number.clone();
    let cache = PILOT_CACHE.lock().await;
    match cache.get(&key) {
        Some(pilot) => Ok(pilot),
        None => {
            if get_replay_status() == ReplayStatus::Replaying {
                let saved = load_replay_pilots();
                info!("Fetching pilot for drone {key} from replay");
                let pilot = saved
                    .get(&key)
                    .context("Pilot not found in replay")?
                    .clone();
                cache.insert(key, pilot.clone()).await;
                return Ok(pilot);
            }
            info!("Fetching pilot details for drone {drone_serial_number}");
            let url = format!("{PILOTS_ENDPOINT}/{drone_serial_number}");
            let response = reqwest::get(url).await?;
            let status = response.status();
            if status.is_success() {
                let json = response.text().await?;
                let pilot: Pilot = serde_json::from_str(&json)?;

                cache.insert(key, pilot.clone()).await;

                std::mem::drop(cache); // avoid deadlocks
                crate::features::replay::save_replay_pilots().await;

                Ok(pilot)
                // without the manual drop, the lock would only be released here
            } else {
                Err(anyhow!(
                    "Reaktor returned an error while fetching pilot details: {} ({})",
                    status.as_u16(),
                    status.canonical_reason().unwrap_or("unknown")
                ))
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, paperclip::actix::Apiv2Schema)]
pub struct Pilot {
    #[serde(alias = "pilotId")]
    pub pilot_id: String,
    #[serde(alias = "firstName")]
    pub first_name: String,
    #[serde(alias = "lastName")]
    pub last_name: String,
    #[serde(alias = "phoneNumber")]
    pub phone_number: String,
    #[serde(alias = "createdDt")]
    pub created_date: String,
    pub email: String,
}
