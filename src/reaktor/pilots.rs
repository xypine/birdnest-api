use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::PILOTS_ENDPOINT;

#[derive(Debug, Serialize, Deserialize)]
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

pub async fn get_pilot(drone_serial_number: &String) -> Result<Pilot> {
    let url = format!("{PILOTS_ENDPOINT}/{drone_serial_number}");
    let response = reqwest::get(url).await?;

    let json = response.text().await?;

    let doc = serde_json::from_str(&json)?;

    Ok(doc)
}
