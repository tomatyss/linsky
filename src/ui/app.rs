//! Main application wrapper for the Linksy email client UI.
//! 
//! This module provides a simple wrapper around the new modular structure.
//! It's kept for backward compatibility and to ease the transition to the new structure.

use crate::controller::AppController;
use crate::state::AppState;
use crate::ui::{init_terminal, restore_terminal, wait_for_key};
use crate::ui::renderer::AppRenderer;
use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

/// Represents the main application.
pub struct App {
    /// Application state
    state: Arc<Mutex<AppState>>,
    /// Application controller
    controller: Arc<AppController>,
    /// Application renderer
    renderer: AppRenderer,
}

impl App {
    /// Creates a new App instance.
    ///
    /// # Returns
    /// A Result containing the App or an error
    pub fn new() -> Result<Self> {
        // Determine base directory
        let base_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?
            .join("linksy");
            
        // Create directories if they don't exist
        std::fs::create_dir_all(&base_dir)?;
        std::fs::create_dir_all(base_dir.join("storage"))?;
        
        // Create configuration manager
        let config_path = base_dir.join("config.json");
        let config_manager = crate::config::ConfigManager::new(config_path.to_str().unwrap())?;
        
        // Create storage
        let storage_path = base_dir.join("storage");
        let storage = crate::storage::EmailStorage::new(&storage_path)?;
        
        // Create state
        let app_state = crate::state::AppState::new(config_manager, storage, base_dir.clone());
        let app_state = Arc::new(Mutex::new(app_state));
        
        // Create managers
        let account_manager = crate::state::AccountManager::new();
        let account_manager = Arc::new(Mutex::new(account_manager));
        
        // Create storage
        let storage_path = base_dir.join("storage");
        let storage = crate::storage::EmailStorage::new(&storage_path)?;
        let email_manager = crate::state::EmailManager::new(storage);
        let email_manager = Arc::new(Mutex::new(email_manager));
        
        // Create controller
        let app_controller = crate::controller::AppController::new(
            app_state.clone(),
            account_manager.clone(),
            email_manager.clone(),
        );
        let app_controller = Arc::new(app_controller);
        
        // Create renderer
        let app_renderer = crate::ui::renderer::AppRenderer::new();
        
        Ok(Self {
            state: app_state,
            controller: app_controller,
            renderer: app_renderer,
        })
    }
    
    /// Runs the application.
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub async fn run(&mut self) -> Result<()> {
        // Initialize the application
        self.controller.initialize().await?;
        
        // Initialize terminal
        let mut terminal = init_terminal()?;
        
        // Main event loop
        while self.state.lock().await.is_running() {
            // Draw UI
            let state = self.state.lock().await;
            terminal.draw(|f| {
                if let Err(e) = self.renderer.render(f, &state) {
                    eprintln!("Error rendering UI: {}", e);
                }
            })?;
            drop(state);
            
            // Handle input
            if let Some(key) = wait_for_key(Some(Duration::from_millis(100)))? {
                let mut state = self.state.lock().await;
                if let Err(e) = crate::controller::InputHandler::new(self.controller.clone())
                    .handle_key(key, &mut state).await {
                    eprintln!("Error handling input: {}", e);
                }
            }
        }
        
        // Disconnect clients
        self.controller.disconnect_all_clients().await?;
        
        // Restore terminal
        restore_terminal(terminal)?;
        
        Ok(())
    }
}
