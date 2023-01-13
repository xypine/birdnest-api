use log::info;
use std::time::Duration;

// Import core functionality from lib.rs
use birdnest_api::prelude::{record_infringements, server};

// Tokio is used as the async runtime
#[tokio::main]
async fn main() {
    // Enable fancier logging
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();
    // Fetch infringements in the background
    let background_task = tokio::spawn(async {
        info!("Background task started!");
        loop {
            tokio::spawn(async {
                record_infringements()
                    .await
                    .expect("Failed to update infringements");
            });
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    });
    // Start the api
    server::start()
        .await
        .expect("Failed to start the api server");
    // Continues once the server has stopped
    info!("The server has stopped, stopping the background task...");
    background_task.abort();
    info!("Everything done, bye!")
}
