use std::collections::HashMap;

use log::debug;
use paperclip::actix::Apiv2Schema;
use serde::Serialize;

use crate::{
    cache::{LATEST_DRONE_SNAPSHOT, PILOT_CACHE},
    reaktor::{drones::DronesDocument, pilots::Pilot},
};

const SAVE_DIR: &str = "replay";

pub fn ensure_dir_exists() {
    let path = std::path::Path::new(SAVE_DIR);
    if !path.exists() {
        std::fs::create_dir(path).unwrap();
    }
}

#[derive(Serialize, Debug, Apiv2Schema, PartialEq, Clone, Copy)]
pub enum ReplayStatus {
    None,
    Recording,
    Replaying,
}

pub fn get_replay_status() -> ReplayStatus {
    let replay =
        std::env::args().any(|a| a == "--replay") || std::env::var("BIRDNEST_REPLAY").is_ok();
    let record = std::env::args().any(|a| a == "--record");
    if replay && record {
        panic!("Cannot replay and record at the same time, remove either --replay, BIRDNEST_REPLAY or --record");
    }
    if replay {
        ReplayStatus::Replaying
    } else if record {
        ReplayStatus::Recording
    } else {
        ReplayStatus::None
    }
}

pub async fn save(time: chrono::DateTime<chrono::Utc>) {
    if get_replay_status() == ReplayStatus::Recording {
        save_replay_drones(time).await;
        save_replay_pilots().await;
    }
}

pub async fn save_replay_pilots() {
    debug!("Saving pilots");
    let pilots_existing = load_replay_pilots();
    debug!("{} pilots in fs", pilots_existing.len());
    let mut pilots = pilots_existing.clone();
    let cache = PILOT_CACHE.lock().await.clone();
    debug!("{} pilots in cache", cache.entry_count());
    for (drone_id, pilot) in &cache {
        pilots.insert((*drone_id).clone(), pilot.clone());
    }
    std::mem::drop(cache);
    if pilots == pilots_existing {
        debug!("No changes to pilots, skipping");
        return;
    } else {
        ensure_dir_exists();
        let path = std::path::Path::new(SAVE_DIR).join("pilots.json");
        debug!("Saving pilot document to {}", path.display());
        std::fs::write(path, serde_json::to_string(&pilots).unwrap()).unwrap();
    }
}

pub(crate) fn load_replay_pilots() -> HashMap<String, Pilot> {
    ensure_dir_exists();
    let path = std::path::Path::new(SAVE_DIR).join("pilots.json");
    if path.exists() {
        let content = std::fs::read_to_string(path).unwrap();
        serde_json::from_str(&content).unwrap()
    } else {
        HashMap::new()
    }
}

async fn save_replay_drones(time: chrono::DateTime<chrono::Utc>) {
    let old_drone_docuents = load_replay_drones().unwrap_or_default();
    let drones = LATEST_DRONE_SNAPSHOT.lock().await;
    if let Some(drones) = &*drones {
        if let Some(last_document) = old_drone_docuents.last() {
            if last_document == &*drones {
                debug!("No changes to drones, skipping");
                return;
            }
        }
        ensure_dir_exists();
        let filename = format!("drones-{}.xml", time.timestamp());
        debug!("Saving drone document to {}", filename);
        let path = std::path::Path::new(SAVE_DIR).join(filename);
        std::fs::write(path, quick_xml::se::to_string(&*drones).unwrap()).unwrap();
    } else {
        debug!("No drones, skipping");
        return;
    }
}

pub(crate) fn load_replay_drones() -> Option<Vec<DronesDocument>> {
    ensure_dir_exists();
    // get all files in the replay directory beginning with "drones"
    let mut files = std::fs::read_dir(SAVE_DIR)
        .ok()?
        .filter_map(|f| f.ok())
        .filter(|f| f.file_name().to_str().unwrap().starts_with("drones"))
        .collect::<Vec<_>>();
    // sort by filename
    files.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
    // read all files
    let mut documents = vec![];
    for file in files {
        let content = std::fs::read_to_string(file.path()).ok()?;
        let doc: DronesDocument = quick_xml::de::from_str(&content).ok()?;
        documents.push(doc);
    }

    Some(documents)
}
