//! Connection management for the Linksy email client.

use crate::models::{Account, ConnectionStatus};
use crate::protocols::{ImapClient, Pop3Client, SmtpClient};
use anyhow::Result;
use log::{error, info};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{Duration, interval};

/// Manages connections to email servers.
pub struct ConnectionManager {
    /// Active accounts
    accounts: Vec<Arc<Mutex<Account>>>,
    /// IMAP clients for each account
    imap_clients: Vec<Arc<Mutex<ImapClient>>>,
    /// POP3 clients for each account
    pop3_clients: Vec<Arc<Mutex<Pop3Client>>>,
    /// SMTP clients for each account
    smtp_clients: Vec<Arc<Mutex<SmtpClient>>>,
    /// Whether the background checker is running
    is_running: Arc<Mutex<bool>>,
    /// Check interval in seconds
    check_interval: u64,
}

impl ConnectionManager {
    /// Creates a new ConnectionManager instance.
    ///
    /// # Parameters
    /// - `accounts`: The active accounts
    /// - `imap_clients`: The IMAP clients
    /// - `pop3_clients`: The POP3 clients
    /// - `smtp_clients`: The SMTP clients
    /// - `check_interval`: The check interval in seconds
    ///
    /// # Returns
    /// A new ConnectionManager instance
    pub fn new(
        accounts: Vec<Arc<Mutex<Account>>>,
        imap_clients: Vec<Arc<Mutex<ImapClient>>>,
        pop3_clients: Vec<Arc<Mutex<Pop3Client>>>,
        smtp_clients: Vec<Arc<Mutex<SmtpClient>>>,
        check_interval: u64,
    ) -> Self {
        Self {
            accounts,
            imap_clients,
            pop3_clients,
            smtp_clients,
            is_running: Arc::new(Mutex::new(false)),
            check_interval,
        }
    }
    
    /// Starts the background connection checker.
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub async fn start_background_checker(&self) -> Result<()> {
        let mut is_running = self.is_running.lock().await;
        if *is_running {
            return Ok(());
        }
        
        *is_running = true;
        drop(is_running);
        
        let accounts = self.accounts.clone();
        let imap_clients = self.imap_clients.clone();
        let pop3_clients = self.pop3_clients.clone();
        let smtp_clients = self.smtp_clients.clone();
        let is_running = self.is_running.clone();
        let check_interval = self.check_interval;
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(check_interval));
            
            loop {
                interval.tick().await;
                
                let running = *is_running.lock().await;
                if !running {
                    break;
                }
                
                // Check connections for each account
                for i in 0..accounts.len() {
                    let account = &accounts[i];
                    let imap_client = &imap_clients[i];
                    let pop3_client = &pop3_clients[i];
                    let smtp_client = &smtp_clients[i];
                    
                    // Check IMAP connection
                    let has_imap = {
                        let account_lock = account.lock().await;
                        account_lock.has_imap()
                    };
                    
                    if has_imap {
                        let imap_status = {
                            let account_lock = account.lock().await;
                            account_lock.imap_status
                        };
                        
                        if imap_status == ConnectionStatus::Connected {
                            // Check if connection is still alive
                            let client = imap_client.lock().await;
                            if !client.is_connected().await {
                                // Connection lost, update status
                                let mut account_lock = account.lock().await;
                                account_lock.imap_status = ConnectionStatus::Disconnected;
                                error!("IMAP connection lost for account {}", account_lock.config.id);
                            }
                        } else if imap_status == ConnectionStatus::Disconnected {
                            // Try to reconnect
                            let mut client = imap_client.lock().await;
                            if let Err(e) = client.connect().await {
                                error!("Failed to reconnect to IMAP: {}", e);
                            } else {
                                let mut account_lock = account.lock().await;
                                account_lock.imap_status = ConnectionStatus::Connected;
                                info!("Reconnected to IMAP for account {}", account_lock.config.id);
                            }
                        }
                    }
                    
                    // Check POP3 connection
                    let has_pop3 = {
                        let account_lock = account.lock().await;
                        account_lock.has_pop3()
                    };
                    
                    if has_pop3 {
                        let pop3_status = {
                            let account_lock = account.lock().await;
                            account_lock.pop3_status
                        };
                        
                        if pop3_status == ConnectionStatus::Disconnected {
                            // Try to reconnect
                            let mut client = pop3_client.lock().await;
                            if let Err(e) = client.connect().await {
                                error!("Failed to reconnect to POP3: {}", e);
                            } else {
                                let mut account_lock = account.lock().await;
                                account_lock.pop3_status = ConnectionStatus::Connected;
                                info!("Reconnected to POP3 for account {}", account_lock.config.id);
                            }
                        }
                    }
                    
                    // Check SMTP connection
                    let smtp_status = {
                        let account_lock = account.lock().await;
                        account_lock.smtp_status
                    };
                    
                    if smtp_status == ConnectionStatus::Disconnected {
                        // Try to reconnect
                        let mut client = smtp_client.lock().await;
                        if let Err(e) = client.connect().await {
                            error!("Failed to reconnect to SMTP: {}", e);
                        } else {
                            let mut account_lock = account.lock().await;
                            account_lock.smtp_status = ConnectionStatus::Connected;
                            info!("Reconnected to SMTP for account {}", account_lock.config.id);
                        }
                    }
                }
            }
        });
        
        Ok(())
    }
    
    /// Stops the background connection checker.
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub async fn stop_background_checker(&self) -> Result<()> {
        let mut is_running = self.is_running.lock().await;
        *is_running = false;
        
        Ok(())
    }
    
    /// Updates the accounts and clients.
    ///
    /// # Parameters
    /// - `accounts`: The active accounts
    /// - `imap_clients`: The IMAP clients
    /// - `pop3_clients`: The POP3 clients
    /// - `smtp_clients`: The SMTP clients
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub async fn update_accounts(
        &mut self,
        accounts: Vec<Arc<Mutex<Account>>>,
        imap_clients: Vec<Arc<Mutex<ImapClient>>>,
        pop3_clients: Vec<Arc<Mutex<Pop3Client>>>,
        smtp_clients: Vec<Arc<Mutex<SmtpClient>>>,
    ) -> Result<()> {
        self.accounts = accounts;
        self.imap_clients = imap_clients;
        self.pop3_clients = pop3_clients;
        self.smtp_clients = smtp_clients;
        
        Ok(())
    }
    
    /// Sets the check interval.
    ///
    /// # Parameters
    /// - `interval`: The check interval in seconds
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub async fn set_check_interval(&mut self, interval: u64) -> Result<()> {
        self.check_interval = interval;
        
        Ok(())
    }
}
