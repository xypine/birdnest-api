use anyhow::{anyhow, Result};
use log::{info, warn};
use serde::{Deserialize, Serialize};

use crate::features::replay::{get_replay_status, load_replay_drones, ReplayStatus};

use super::DRONES_ENDPOINT;

pub async fn get_drones() -> Result<DronesDocument> {
    if get_replay_status() == ReplayStatus::Replaying {
        let history = load_replay_drones().unwrap();
        let unix_now = chrono::Utc::now().timestamp();
        let history_len = history.len();
        if history_len == 0 {
            return Err(anyhow!("No drones in replay history"));
        }
        // We'll just assume this for now
        let seconds_between_updates = 2;
        // We use this to get a deterministic index that is continous over time
        let index = ((unix_now / seconds_between_updates) % history_len as i64) as usize;
        info!("Replaying drones from index {} / {}", index, history_len);
        if index == 0 {
            warn!("Replaying drones from the beginning, invalidating all previous infringements");
            let infringements = crate::INFRINGEMENTS.lock().await;
            infringements.invalidate_all();
        }
        let doc = history[index].clone();
        *crate::cache::LATEST_DRONE_SNAPSHOT.lock().await = Some(doc.clone());
        return Ok(doc);
    }
    let response = reqwest::get(DRONES_ENDPOINT).await?;
    let status = response.status();
    if status.is_success() {
        let xml = response.text().await?;
        let doc: DronesDocument = quick_xml::de::from_str(&xml)?;

        *crate::cache::LATEST_DRONE_SNAPSHOT.lock().await = Some(doc.clone());
        crate::features::replay::save(chrono::Utc::now()).await;

        Ok(doc)
    } else {
        Err(anyhow!(
            "Reaktor returned an error while fetching drones: {} ({})",
            status.as_u16(),
            status.canonical_reason().unwrap_or("unknown")
        ))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DronesDocument {
    #[serde(alias = "deviceInformation")]
    pub device_information: DronesSensorInfo,
    pub capture: DronesCapture,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DronesSensorInfo {
    #[serde(alias = "@deviceId")]
    pub device_id: Option<String>,

    #[serde(alias = "listenRange")]
    pub listen_range: Option<usize>,
    #[serde(alias = "deviceStarted")]
    pub device_started: Option<String>,
    #[serde(alias = "uptimeSeconds")]
    pub uptime_seconds: Option<usize>,
    #[serde(alias = "updateIntervalMs")]
    pub update_interval_ms: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DronesCapture {
    #[serde(alias = "@snapshotTimestamp")]
    pub snapshot_timestamp: String,
    pub drone: Vec<Drone>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Drone {
    #[serde(alias = "serialNumber")]
    pub serial_number: String,
    pub model: String,
    pub manufacturer: String,
    pub mac: String,
    pub ipv4: String,
    pub ipv6: String,
    pub firmware: String,
    #[serde(alias = "positionY")]
    pub position_y: f64,
    #[serde(alias = "positionX")]
    pub position_x: f64,
    pub altitude: f64,
}
