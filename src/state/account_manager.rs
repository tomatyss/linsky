//! Account management for the Linksy email client.

use crate::config::{EmailAccount, ConfigManager};
use crate::models::{Account, ConnectionStatus};
use crate::protocols::{ImapClient, Pop3Client, SmtpClient};
use anyhow::Result;
use log::error;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Manages email accounts and their connections.
pub struct AccountManager {
    /// Active accounts
    accounts: Vec<Arc<Mutex<Account>>>,
    /// IMAP clients for each account
    imap_clients: Vec<Arc<Mutex<ImapClient>>>,
    /// POP3 clients for each account
    pop3_clients: Vec<Arc<Mutex<Pop3Client>>>,
    /// SMTP clients for each account
    smtp_clients: Vec<Arc<Mutex<SmtpClient>>>,
}

impl AccountManager {
    /// Creates a new AccountManager instance.
    ///
    /// # Returns
    /// A new AccountManager instance
    pub fn new() -> Self {
        Self {
            accounts: Vec::new(),
            imap_clients: Vec::new(),
            pop3_clients: Vec::new(),
            smtp_clients: Vec::new(),
        }
    }
    
    /// Loads accounts from configuration.
    ///
    /// # Parameters
    /// - `config_manager`: The configuration manager
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub async fn load_accounts(&mut self, config_manager: &ConfigManager) -> Result<()> {
        // Clear existing accounts and clients
        self.accounts.clear();
        self.imap_clients.clear();
        self.pop3_clients.clear();
        self.smtp_clients.clear();
        
        // Get accounts from configuration
        let config = config_manager.get_config();
        
        // Create account objects
        for account_config in &config.accounts {
            let account = Account::new(account_config.clone());
            let account_arc = Arc::new(Mutex::new(account));
            
            // Create protocol clients
            let imap_client = Arc::new(Mutex::new(ImapClient::new(account_arc.clone())));
            let pop3_client = Arc::new(Mutex::new(Pop3Client::new(account_arc.clone())));
            let smtp_client = Arc::new(Mutex::new(SmtpClient::new(account_arc.clone())));
            
            // Store clients
            self.accounts.push(account_arc);
            self.imap_clients.push(imap_client);
            self.pop3_clients.push(pop3_client);
            self.smtp_clients.push(smtp_client);
        }
        
        Ok(())
    }
    
    /// Gets the accounts.
    ///
    /// # Returns
    /// A reference to the accounts
    pub fn get_accounts(&self) -> &Vec<Arc<Mutex<Account>>> {
        &self.accounts
    }
    
    /// Gets a specific account.
    ///
    /// # Parameters
    /// - `index`: The account index
    ///
    /// # Returns
    /// An Option containing a reference to the account
    pub fn get_account(&self, index: usize) -> Option<&Arc<Mutex<Account>>> {
        self.accounts.get(index)
    }
    
    /// Connects to the specified account.
    ///
    /// # Parameters
    /// - `index`: The account index
    ///
    /// # Returns
    /// A Result containing a boolean indicating if any connection was successful
    pub async fn connect_account(&self, index: usize) -> Result<bool> {
        if index >= self.accounts.len() {
            return Ok(false);
        }
        
        // Track if any connection was successful
        let mut any_connection_successful = false;
        
        // Check if account has IMAP
        let has_imap = {
            let account = &self.accounts[index];
            account.lock().await.has_imap()
        };
        
        if has_imap {
            // Connect to IMAP
            let result = {
                let imap_client = &self.imap_clients[index];
                let mut client = imap_client.lock().await;
                client.connect().await
            };
            
            // Handle result
            if let Err(e) = result {
                let error_msg = format!("IMAP connection failed: {}", e);
                {
                    let account = &self.accounts[index];
                    let mut account_lock = account.lock().await;
                    account_lock.imap_status = ConnectionStatus::Failed;
                    account_lock.last_error = Some(e.to_string());
                }
                error!("{}", error_msg);
            } else {
                any_connection_successful = true;
            }
        }
        
        // Check if account has POP3
        let has_pop3 = {
            let account = &self.accounts[index];
            account.lock().await.has_pop3()
        };
        
        if has_pop3 {
            // Connect to POP3
            let result = {
                let pop3_client = &self.pop3_clients[index];
                let mut client = pop3_client.lock().await;
                client.connect().await
            };
            
            // Handle result
            if let Err(e) = result {
                let error_msg = format!("POP3 connection failed: {}", e);
                {
                    let account = &self.accounts[index];
                    let mut account_lock = account.lock().await;
                    account_lock.pop3_status = ConnectionStatus::Failed;
                    account_lock.last_error = Some(e.to_string());
                }
                error!("{}", error_msg);
            } else {
                any_connection_successful = true;
            }
        }
        
        // Connect to SMTP
        let result = {
            let smtp_client = &self.smtp_clients[index];
            let mut client = smtp_client.lock().await;
            client.connect().await
        };
        
        // Handle result
        if let Err(e) = result {
            let error_msg = format!("SMTP connection failed: {}", e);
            {
                let account = &self.accounts[index];
                let mut account_lock = account.lock().await;
                account_lock.smtp_status = ConnectionStatus::Failed;
                account_lock.last_error = Some(e.to_string());
            }
            error!("{}", error_msg);
        } else {
            any_connection_successful = true;
        }
        
        Ok(any_connection_successful)
    }
    
    /// Disconnects all clients.
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub async fn disconnect_all_clients(&self) -> Result<()> {
        for i in 0..self.accounts.len() {
            let imap_client = &self.imap_clients[i];
            let pop3_client = &self.pop3_clients[i];
            let smtp_client = &self.smtp_clients[i];
            
            // Disconnect IMAP
            let mut client = imap_client.lock().await;
            if let Err(e) = client.disconnect().await {
                error!("IMAP disconnect failed: {}", e);
            }
            
            // Disconnect POP3
            let mut client = pop3_client.lock().await;
            if let Err(e) = client.disconnect().await {
                error!("POP3 disconnect failed: {}", e);
            }
            
            // Disconnect SMTP
            let mut client = smtp_client.lock().await;
            if let Err(e) = client.disconnect().await {
                error!("SMTP disconnect failed: {}", e);
            }
        }
        
        Ok(())
    }
    
    /// Attempts to retry connecting to failed services for the specified account.
    ///
    /// # Parameters
    /// - `index`: The account index
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub async fn retry_connections(&self, index: usize) -> Result<()> {
        if index >= self.accounts.len() {
            return Ok(());
        }
        
        // Check and retry IMAP if needed
        let retry_imap = {
            let account = &self.accounts[index];
            let account_lock = account.lock().await;
            account_lock.has_imap() && account_lock.imap_status == ConnectionStatus::Failed
        };
        
        if retry_imap {
            // Retry IMAP connection
            let result = {
                let imap_client = &self.imap_clients[index];
                let mut client = imap_client.lock().await;
                client.connect().await
            };
            
            // Handle result
            if let Err(e) = result {
                let error_msg = format!("IMAP connection retry failed: {}", e);
                {
                    let account = &self.accounts[index];
                    let mut account_lock = account.lock().await;
                    account_lock.last_error = Some(e.to_string());
                }
                error!("{}", error_msg);
            }
        }
        
        // Check and retry POP3 if needed
        let retry_pop3 = {
            let account = &self.accounts[index];
            let account_lock = account.lock().await;
            account_lock.has_pop3() && account_lock.pop3_status == ConnectionStatus::Failed
        };
        
        if retry_pop3 {
            // Retry POP3 connection
            let result = {
                let pop3_client = &self.pop3_clients[index];
                let mut client = pop3_client.lock().await;
                client.connect().await
            };
            
            // Handle result
            if let Err(e) = result {
                let error_msg = format!("POP3 connection retry failed: {}", e);
                {
                    let account = &self.accounts[index];
                    let mut account_lock = account.lock().await;
                    account_lock.last_error = Some(e.to_string());
                }
                error!("{}", error_msg);
            }
        }
        
        // Check and retry SMTP if needed
        let retry_smtp = {
            let account = &self.accounts[index];
            let account_lock = account.lock().await;
            account_lock.smtp_status == ConnectionStatus::Failed
        };
        
        if retry_smtp {
            // Retry SMTP connection
            let result = {
                let smtp_client = &self.smtp_clients[index];
                let mut client = smtp_client.lock().await;
                client.connect().await
            };
            
            // Handle result
            if let Err(e) = result {
                let error_msg = format!("SMTP connection retry failed: {}", e);
                {
                    let account = &self.accounts[index];
                    let mut account_lock = account.lock().await;
                    account_lock.last_error = Some(e.to_string());
                }
                error!("{}", error_msg);
            }
        }
        
        Ok(())
    }
    
    /// Gets the IMAP client for the specified account.
    ///
    /// # Parameters
    /// - `index`: The account index
    ///
    /// # Returns
    /// An Option containing a reference to the IMAP client
    pub fn get_imap_client(&self, index: usize) -> Option<&Arc<Mutex<ImapClient>>> {
        self.imap_clients.get(index)
    }
    
    /// Gets the POP3 client for the specified account.
    ///
    /// # Parameters
    /// - `index`: The account index
    ///
    /// # Returns
    /// An Option containing a reference to the POP3 client
    pub fn get_pop3_client(&self, index: usize) -> Option<&Arc<Mutex<Pop3Client>>> {
        self.pop3_clients.get(index)
    }
    
    /// Gets the SMTP client for the specified account.
    ///
    /// # Parameters
    /// - `index`: The account index
    ///
    /// # Returns
    /// An Option containing a reference to the SMTP client
    pub fn get_smtp_client(&self, index: usize) -> Option<&Arc<Mutex<SmtpClient>>> {
        self.smtp_clients.get(index)
    }
    
    /// Adds a new account.
    ///
    /// # Parameters
    /// - `account_config`: The account configuration
    /// - `config_manager`: The configuration manager
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub async fn add_account(&mut self, account_config: EmailAccount, config_manager: &mut ConfigManager) -> Result<()> {
        // Add account to configuration
        let config = config_manager.get_config_mut();
        config.accounts.push(account_config.clone());
        
        // Save configuration
        config_manager.save_config()?;
        
        // Create account object
        let account = Account::new(account_config);
        let account_arc = Arc::new(Mutex::new(account));
        
        // Create protocol clients
        let imap_client = Arc::new(Mutex::new(ImapClient::new(account_arc.clone())));
        let pop3_client = Arc::new(Mutex::new(Pop3Client::new(account_arc.clone())));
        let smtp_client = Arc::new(Mutex::new(SmtpClient::new(account_arc.clone())));
        
        // Store clients
        self.accounts.push(account_arc);
        self.imap_clients.push(imap_client);
        self.pop3_clients.push(pop3_client);
        self.smtp_clients.push(smtp_client);
        
        Ok(())
    }
    
    /// Updates an existing account.
    ///
    /// # Parameters
    /// - `index`: The account index
    /// - `account_config`: The updated account configuration
    /// - `config_manager`: The configuration manager
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub async fn update_account(&mut self, index: usize, account_config: EmailAccount, config_manager: &mut ConfigManager) -> Result<()> {
        if index >= self.accounts.len() {
            return Ok(());
        }
        
        // Update account in configuration
        let config = config_manager.get_config_mut();
        if let Some(existing_config) = config.accounts.get_mut(index) {
            *existing_config = account_config.clone();
        } else {
            return Ok(());
        }
        
        // Save configuration
        config_manager.save_config()?;
        
        // Disconnect existing clients
        {
            let imap_client = &self.imap_clients[index];
            let mut client = imap_client.lock().await;
            if let Err(e) = client.disconnect().await {
                error!("IMAP disconnect failed: {}", e);
            }
        }
        
        {
            let pop3_client = &self.pop3_clients[index];
            let mut client = pop3_client.lock().await;
            if let Err(e) = client.disconnect().await {
                error!("POP3 disconnect failed: {}", e);
            }
        }
        
        {
            let smtp_client = &self.smtp_clients[index];
            let mut client = smtp_client.lock().await;
            if let Err(e) = client.disconnect().await {
                error!("SMTP disconnect failed: {}", e);
            }
        }
        
        // Update account object
        {
            let account = &self.accounts[index];
            let mut account_lock = account.lock().await;
            *account_lock = Account::new(account_config);
        }
        
        Ok(())
    }
    
    /// Deletes an account.
    ///
    /// # Parameters
    /// - `index`: The account index
    /// - `config_manager`: The configuration manager
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub async fn delete_account(&mut self, index: usize, config_manager: &mut ConfigManager) -> Result<()> {
        if index >= self.accounts.len() {
            return Ok(());
        }
        
        // Get account ID
        let account_id = {
            let account = &self.accounts[index];
            let account_lock = account.lock().await;
            account_lock.config.id.clone()
        };
        
        // Disconnect clients
        {
            let imap_client = &self.imap_clients[index];
            let mut client = imap_client.lock().await;
            if let Err(e) = client.disconnect().await {
                error!("IMAP disconnect failed: {}", e);
            }
        }
        
        {
            let pop3_client = &self.pop3_clients[index];
            let mut client = pop3_client.lock().await;
            if let Err(e) = client.disconnect().await {
                error!("POP3 disconnect failed: {}", e);
            }
        }
        
        {
            let smtp_client = &self.smtp_clients[index];
            let mut client = smtp_client.lock().await;
            if let Err(e) = client.disconnect().await {
                error!("SMTP disconnect failed: {}", e);
            }
        }
        
        // Remove account from configuration
        let config = config_manager.get_config_mut();
        config.accounts.remove(index);
        
        // If the removed account was the default, clear the default
        if let Some(default_id) = &config.settings.default_account {
            if default_id == &account_id {
                config.settings.default_account = None;
            }
        }
        
        // Save configuration
        config_manager.save_config()?;
        
        // Remove clients
        self.accounts.remove(index);
        self.imap_clients.remove(index);
        self.pop3_clients.remove(index);
        self.smtp_clients.remove(index);
        
        Ok(())
    }
}

impl Default for AccountManager {
    fn default() -> Self {
        Self::new()
    }
}
