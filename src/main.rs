mod controller;
mod gui;
mod network;
mod models;
mod arm;
mod plot;

use controller::Controller;
use gui::Gui;
use network::Network;
use models::SharedState; // Use this if you have a separate models.rs
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;

fn main() -> Result<(), eframe::Error> {
    env_logger::init();

    // Shared state initialization
    let shared_state = Arc::new(Mutex::new(SharedState::new("_controller._udp.local".to_owned())));

    // Create a Tokio runtime
    let rt = Runtime::new().expect("Failed to create Tokio runtime");

    // Clone the shared state for the async network task
    let state_for_async = shared_state.clone();

    // Spawn the mDNS discovery task using the network module
    rt.spawn(async move {
        Network::discover_devices(state_for_async).await;
    });

    // Set eframe options if required
    let options = eframe::NativeOptions::default();

    // Run the eframe application, initializing Gui with the shared state
    eframe::run_native(
        "Robotic Limb Controller",
        options,
        Box::new(move |cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Box::new(Gui::new(shared_state.clone()))
        }),
    )
}