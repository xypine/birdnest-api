use lazy_static::lazy_static;
use moka::future::Cache as GenericCache;
use tokio::sync::Mutex;

use crate::{
    config::INFRINGEMENT_DURATION,
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
            // Infringements are automatically deleted after [INFRINGEMENT_DURATION]
            .time_to_live(INFRINGEMENT_DURATION)
            .build()
    );
}
