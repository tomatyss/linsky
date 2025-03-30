//! Configuration management for the Linksy email client.
//! 
//! This module handles loading, saving, and accessing user configuration
//! including email accounts, server settings, and application preferences.

use anyhow::Result;
use config::{Config, File};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Represents the application configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Email accounts configured in the application
    pub accounts: Vec<EmailAccount>,
    /// General application settings
    pub settings: AppSettings,
}

/// Represents an email account configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailAccount {
    /// Unique identifier for the account
    pub id: String,
    /// User-friendly name for the account
    pub name: String,
    /// Email address
    pub email: String,
    /// IMAP server configuration
    pub imap: Option<ServerConfig>,
    /// POP3 server configuration
    pub pop3: Option<ServerConfig>,
    /// SMTP server configuration
    pub smtp: ServerConfig,
}

/// Represents a mail server configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server hostname
    pub host: String,
    /// Server port
    pub port: u16,
    /// Username for authentication
    pub username: String,
    /// Password for authentication
    pub password: String,
    /// Whether to use SSL/TLS
    pub use_ssl: bool,
}

/// Represents general application settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// Default account ID to use
    pub default_account: Option<String>,
    /// Whether to check for new emails automatically
    pub auto_check: bool,
    /// Interval in minutes for auto-checking emails
    pub check_interval: u32,
}

/// Configuration manager for the application.
#[derive(Clone)]
pub struct ConfigManager {
    config: AppConfig,
    #[allow(dead_code)]
    config_path: String,
}

impl ConfigManager {
    /// Creates a new ConfigManager instance.
    ///
    /// # Parameters
    /// - `config_path`: Path to the configuration file
    ///
    /// # Returns
    /// A Result containing the ConfigManager or an error
    pub fn new(config_path: &str) -> Result<Self> {
        let config = Self::load_config(config_path)?;
        
        Ok(Self {
            config,
            config_path: config_path.to_string(),
        })
    }
    
    /// Loads the configuration from the specified path.
    ///
    /// # Parameters
    /// - `config_path`: Path to the configuration file
    ///
    /// # Returns
    /// A Result containing the AppConfig or an error
    fn load_config(config_path: &str) -> Result<AppConfig> {
        let path = Path::new(config_path);
        
        // If config file doesn't exist, create a default config
        if !path.exists() {
            return Ok(Self::default_config());
        }
        
        let config = Config::builder()
            .add_source(File::with_name(config_path))
            .build()?;
            
        let app_config = config.try_deserialize::<AppConfig>()?;
        Ok(app_config)
    }
    
    /// Creates a default configuration.
    ///
    /// # Returns
    /// A default AppConfig
    fn default_config() -> AppConfig {
        AppConfig {
            accounts: Vec::new(),
            settings: AppSettings {
                default_account: None,
                auto_check: true,
                check_interval: 15,
            },
        }
    }
    
    /// Saves the current configuration to disk.
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub fn save_config(&self) -> Result<()> {
        let serialized = serde_json::to_string_pretty(&self.config)?;
        std::fs::write(&self.config_path, serialized)?;
        Ok(())
    }
    
    /// Saves a specific configuration to disk.
    ///
    /// # Parameters
    /// - `config`: The configuration to save
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub fn save_config_instance(&self, config: &AppConfig) -> Result<()> {
        let serialized = serde_json::to_string_pretty(config)?;
        std::fs::write(&self.config_path, serialized)?;
        Ok(())
    }
    
    /// Gets a reference to the current configuration.
    ///
    /// # Returns
    /// A reference to the AppConfig
    pub fn get_config(&self) -> &AppConfig {
        &self.config
    }
    
    /// Gets a mutable reference to the current configuration.
    ///
    /// # Returns
    /// A mutable reference to the AppConfig
    #[allow(dead_code)]
    pub fn get_config_mut(&mut self) -> &mut AppConfig {
        &mut self.config
    }
    
    /// Adds a new email account to the configuration.
    ///
    /// # Parameters
    /// - `account`: The email account to add
    ///
    /// # Returns
    /// A Result indicating success or failure
    #[allow(dead_code)]
    pub fn add_account(&mut self, account: EmailAccount) -> Result<()> {
        // Check if account with same ID already exists
        if self.config.accounts.iter().any(|a| a.id == account.id) {
            return Err(anyhow::anyhow!("Account with ID {} already exists", account.id));
        }
        
        self.config.accounts.push(account);
        self.save_config()?;
        Ok(())
    }
    
    /// Removes an email account from the configuration.
    ///
    /// # Parameters
    /// - `account_id`: The ID of the account to remove
    ///
    /// # Returns
    /// A Result indicating success or failure
    #[allow(dead_code)]
    pub fn remove_account(&mut self, account_id: &str) -> Result<()> {
        let initial_len = self.config.accounts.len();
        self.config.accounts.retain(|a| a.id != account_id);
        
        if self.config.accounts.len() == initial_len {
            return Err(anyhow::anyhow!("Account with ID {} not found", account_id));
        }
        
        // If the removed account was the default, clear the default
        if let Some(default_id) = &self.config.settings.default_account {
            if default_id == account_id {
                self.config.settings.default_account = None;
            }
        }
        
        self.save_config()?;
        Ok(())
    }
}
