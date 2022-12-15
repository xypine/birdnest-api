use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::DRONES_ENDPOINT;

pub async fn get_drones() -> Result<DronesDocument> {
    let response = reqwest::get(DRONES_ENDPOINT).await?;
    let xml = response.text().await?;
    let doc: DronesDocument = quick_xml::de::from_str(&xml)?;

    *crate::cache::LATEST_DRONE_SNAPSHOT.lock().await = Some(doc.clone());

    Ok(doc)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DronesDocument {
    #[serde(alias = "deviceInformation")]
    pub device_information: DronesSensorInfo,
    pub capture: DronesCapture,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DronesSensorInfo {
    #[serde(alias = "deviceId")]
    pub device_id: String,

    #[serde(alias = "listenRange")]
    pub listen_range: usize,
    #[serde(alias = "deviceStarted")]
    pub device_started: String,
    #[serde(alias = "uptimeSeconds")]
    pub uptime_seconds: usize,
    #[serde(alias = "updateIntervalMs")]
    pub update_interval_ms: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DronesCapture {
    #[serde(alias = "snapshotTimestamp")]
    pub snapshot_timestamp: String,
    pub drone: Vec<Drone>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
