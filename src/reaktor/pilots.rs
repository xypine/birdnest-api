use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::PILOTS_ENDPOINT;

pub async fn get_pilot(drone_serial_number: &String) -> Result<Pilot> {
    let url = format!("{PILOTS_ENDPOINT}/{drone_serial_number}");
    let response = reqwest::get(url).await?;
    let json = response.text().await?;
    let pilot = serde_json::from_str(&json)?;

    Ok(pilot)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
