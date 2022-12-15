use lazy_static::lazy_static;
use moka::future::Cache as GenericCache;
use tokio::sync::Mutex;

use crate::{
    reaktor::{drones::DronesDocument, pilots::Pilot},
    Infringement,
};

pub type PilotCache = GenericCache<String, Pilot>;
pub type InfringementsCache = GenericCache<String, Infringement>;
lazy_static! {
    pub static ref LATEST_DRONE_SNAPSHOT: Mutex<Option<DronesDocument>> = Mutex::new(None);
    pub static ref PILOT_CACHE: Mutex<PilotCache> =
        Mutex::new(PilotCache::builder().max_capacity(10_000).build());
    pub static ref INFRINGEMENTS: Mutex<InfringementsCache> = Mutex::new(
        InfringementsCache::builder()
            .max_capacity(10_000)
            .time_to_live(crate::config::INFRINGEMENT_DURATION)
            .build()
    );
}
