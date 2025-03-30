mod config;
mod controller;
mod models;
mod protocols;
mod state;
mod storage;
mod ui;

use anyhow::Result;
use log::info;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Main entry point for the Linksy email client application.
/// Initializes logging, sets up the application, and starts the UI.
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();
    info!("Starting Linksy email client");

    // Determine base directory
    let base_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?
        .join("linksy");
        
    // Create directories if they don't exist
    std::fs::create_dir_all(&base_dir)?;
    std::fs::create_dir_all(base_dir.join("storage"))?;
    
    // Create configuration manager
    let config_path = base_dir.join("config.json");
    let config_manager = config::ConfigManager::new(config_path.to_str().unwrap())?;
    
    // Create a single storage instance
    let storage_path = base_dir.join("storage");
    let storage = storage::EmailStorage::new(&storage_path)?;
    
    // Create state
    let app_state = state::AppState::new(config_manager, storage.clone(), base_dir);
    let app_state = Arc::new(Mutex::new(app_state));
    
    // Create managers
    let account_manager = state::AccountManager::new();
    let account_manager = Arc::new(Mutex::new(account_manager));
    
    // Use a clone of the storage instance for the email manager
    let email_manager = state::EmailManager::new(storage);
    let email_manager = Arc::new(Mutex::new(email_manager));
    
    // Create controller
    let app_controller = controller::AppController::new(
        app_state.clone(),
        account_manager.clone(),
        email_manager.clone(),
    );
    let app_controller = Arc::new(app_controller);
    
    // Create input handler
    let input_handler = controller::InputHandler::new(app_controller.clone());
    
    // Create renderer
    let app_renderer = ui::renderer::AppRenderer::new();
    
    // Initialize the application
    app_controller.initialize().await?;
    
    // Initialize terminal
    let mut terminal = ui::init_terminal()?;
    
    // Main event loop
    while app_state.lock().await.is_running() {
        // Draw UI
        let state = app_state.lock().await;
        terminal.draw(|f| {
            if let Err(e) = app_renderer.render(f, &state) {
                eprintln!("Error rendering UI: {}", e);
            }
        })?;
        drop(state);
        
        // Handle input
        if let Some(key) = ui::wait_for_key(Some(std::time::Duration::from_millis(100)))? {
            let mut state = app_state.lock().await;
            if let Err(e) = input_handler.handle_key(key, &mut state).await {
                eprintln!("Error handling input: {}", e);
            }
        }
    }
    
    // Shutdown the application
    app_controller.shutdown().await?;
    
    // Restore terminal
    ui::restore_terminal(terminal)?;
    
    info!("Linksy email client shut down successfully");
    Ok(())
}
