mod config;
mod models;
mod protocols;
mod storage;
mod ui;

use anyhow::Result;
use log::info;

/// Main entry point for the Linksy email client application.
/// Initializes logging, sets up the application, and starts the UI.
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();
    info!("Starting Linksy email client");

    // Initialize the UI application
    let mut app = ui::app::App::new()?;
    
    // Run the application
    app.run().await?;

    info!("Linksy email client shutting down");
    Ok(())
}
