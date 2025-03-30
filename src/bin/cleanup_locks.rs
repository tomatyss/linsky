//! Utility to clean up stale lock files for the Linksy email client.

use anyhow::Result;
use log::info;
use std::path::PathBuf;

/// Main entry point for the cleanup utility.
fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();
    info!("Linksy lock file cleanup utility");

    // Determine base directory
    let base_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?
        .join("linksy");
        
    // Check if the directory exists
    if !base_dir.exists() {
        println!("Linksy directory not found at {:?}", base_dir);
        return Ok(());
    }
    
    // Create storage path
    let storage_path = base_dir.join("storage");
    if !storage_path.exists() {
        println!("Storage directory not found at {:?}", storage_path);
        return Ok(());
    }
    
    println!("Checking for stale lock files in {:?}", storage_path);
    
    // Clean up stale lock files
    match linksy::storage::EmailStorage::cleanup_stale_lock_files(&storage_path) {
        Ok(_) => {
            println!("Lock file cleanup completed successfully");
        },
        Err(e) => {
            println!("Error during lock file cleanup: {}", e);
        }
    }
    
    Ok(())
}
