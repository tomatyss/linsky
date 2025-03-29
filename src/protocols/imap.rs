//! IMAP protocol implementation for the Linksy email client.

use crate::config::ServerConfig;
use crate::models::{Account, ConnectionStatus, Email};
use anyhow::{anyhow, Result};
use imap::types::{Flag};
use log::{debug, error};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Represents an IMAP client connection.
pub struct ImapClient {
    /// The account this client is connected to
    account: Arc<Mutex<Account>>,
    /// The IMAP session
    session: Option<Arc<Mutex<imap::Session<native_tls::TlsStream<std::net::TcpStream>>>>>,
}

impl ImapClient {
    /// Creates a new IMAP client for the specified account.
    ///
    /// # Parameters
    /// - `account`: The email account to connect to
    ///
    /// # Returns
    /// A new ImapClient instance
    pub fn new(account: Arc<Mutex<Account>>) -> Self {
        Self {
            account,
            session: None,
        }
    }
    
    /// Connects to the IMAP server.
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub async fn connect(&mut self) -> Result<()> {
        let account = self.account.lock().await;
        
        // Check if IMAP is configured
        let imap_config = account.get_imap_config()
            .ok_or_else(|| anyhow!("IMAP is not configured for this account"))?;
        
        // Clone the config for later use
        let imap_config = imap_config.clone();
        
        // Update connection status
        let mut account = account;
        account.imap_status = ConnectionStatus::Connecting;
        drop(account); // Release the lock
        
        // Connect to the server
        let client = self.create_client(&imap_config).await?;
        
        // Store the session
        self.session = Some(Arc::new(Mutex::new(client)));
        
        // Update connection status
        let mut account = self.account.lock().await;
        account.imap_status = ConnectionStatus::Connected;
        account.last_error = None;
        
        Ok(())
    }
    
    /// Creates an IMAP client and connects to the server.
    ///
    /// # Parameters
    /// - `config`: The server configuration
    ///
    /// # Returns
    /// A Result containing the IMAP session or an error
    async fn create_client(&self, config: &ServerConfig) -> Result<imap::Session<native_tls::TlsStream<std::net::TcpStream>>> {
        // Create TLS connector
        let tls = native_tls::TlsConnector::builder().build()?;
        
        // Connect to the server using TLS
        let tls_stream = tls.connect(&config.host, std::net::TcpStream::connect(format!("{}:{}", config.host, config.port))?)?;
        
        // Create a new client with the TLS stream
        let client = imap::Client::new(tls_stream);
        
        // Login to the server
        let mut imap_session = client.login(&config.username, &config.password)
            .map_err(|e| anyhow!("Login failed: {:?}", e))?;
        
        // List available mailboxes
        let mailboxes = imap_session.list(None, Some("*"))?;
        
        // Update account with available folders
        let mut account = self.account.lock().await;
        account.folders = mailboxes.iter()
            .filter_map(|m| Some(m.name()))
            .map(|n| n.to_string())
            .collect();
            
        // Return the session
        Ok(imap_session)
    }
    
    /// Disconnects from the IMAP server.
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub async fn disconnect(&mut self) -> Result<()> {
        if let Some(session) = &self.session {
            let mut session = session.lock().await;
            session.logout()?;
        }
        
        self.session = None;
        
        let mut account = self.account.lock().await;
        account.imap_status = ConnectionStatus::Disconnected;
        
        Ok(())
    }
    
    /// Fetches emails from the specified mailbox.
    ///
    /// # Parameters
    /// - `mailbox`: The mailbox to fetch emails from
    /// - `limit`: Maximum number of emails to fetch
    ///
    /// # Returns
    /// A Result containing a vector of emails or an error
    pub async fn fetch_emails(&self, mailbox: &str, limit: usize) -> Result<Vec<Email>> {
        let session_arc = self.session.as_ref()
            .ok_or_else(|| anyhow!("Not connected to IMAP server"))?;
            
        let mut session = session_arc.lock().await;
        
        // Select the mailbox
        let mailbox_data = session.select(mailbox)?;
        debug!("Selected mailbox: {} with {} messages", mailbox, mailbox_data.exists);
        
        // Update account with message counts
        let mut account = self.account.lock().await;
        account.total_count = mailbox_data.exists as usize;
        
        // Count unread messages
        let unseen = session.search("UNSEEN")?;
        account.unread_count = unseen.len();
        drop(account); // Release the lock
        
        // Fetch the most recent messages
        let sequence = if mailbox_data.exists > limit as u32 {
            format!("{}:{}", mailbox_data.exists - limit as u32 + 1, mailbox_data.exists)
        } else {
            "1:*".to_string()
        };
        
        let messages = session.fetch(sequence, "(RFC822 FLAGS UID)")?;
        
        // Parse emails
        let mut emails = Vec::new();
        let account = self.account.lock().await;
        
        for message in messages.iter() {
            if let Some(body) = message.body() {
                match Email::parse_from_raw(body, &account.config.id, mailbox) {
                    Ok(mut email) => {
                        // Set flags
                        let flags = message.flags();
                        email.is_read = flags.iter().any(|flag| *flag == Flag::Seen);
                        email.is_flagged = flags.iter().any(|flag| *flag == Flag::Flagged);
                        
                        // Set ID from UID
                        if let Some(uid) = message.uid {
                            email.id = uid.to_string();
                        }
                        
                        emails.push(email);
                    },
                    Err(e) => {
                        error!("Failed to parse email: {}", e);
                    }
                }
            }
        }
        
        // Sort emails by date (newest first)
        emails.sort_by(|a, b| b.date.cmp(&a.date));
        
        Ok(emails)
    }
    
    /// Marks an email as read.
    ///
    /// # Parameters
    /// - `mailbox`: The mailbox containing the email
    /// - `email_id`: The ID of the email to mark
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub async fn mark_as_read(&self, mailbox: &str, email_id: &str) -> Result<()> {
        let session_arc = self.session.as_ref()
            .ok_or_else(|| anyhow!("Not connected to IMAP server"))?;
            
        let mut session = session_arc.lock().await;
        
        // Select the mailbox
        session.select(mailbox)?;
        
        // Mark the message as read
        session.uid_store(email_id, "+FLAGS (\\Seen)")?;
        
        Ok(())
    }
    
    /// Marks an email as unread.
    ///
    /// # Parameters
    /// - `mailbox`: The mailbox containing the email
    /// - `email_id`: The ID of the email to mark
    ///
    /// # Returns
    /// A Result indicating success or failure
    #[allow(dead_code)]
    pub async fn mark_as_unread(&self, mailbox: &str, email_id: &str) -> Result<()> {
        let session_arc = self.session.as_ref()
            .ok_or_else(|| anyhow!("Not connected to IMAP server"))?;
            
        let mut session = session_arc.lock().await;
        
        // Select the mailbox
        session.select(mailbox)?;
        
        // Mark the message as unread
        session.uid_store(email_id, "-FLAGS (\\Seen)")?;
        
        Ok(())
    }
    
    /// Flags an email.
    ///
    /// # Parameters
    /// - `mailbox`: The mailbox containing the email
    /// - `email_id`: The ID of the email to flag
    ///
    /// # Returns
    /// A Result indicating success or failure
    #[allow(dead_code)]
    pub async fn flag_email(&self, mailbox: &str, email_id: &str) -> Result<()> {
        let session_arc = self.session.as_ref()
            .ok_or_else(|| anyhow!("Not connected to IMAP server"))?;
            
        let mut session = session_arc.lock().await;
        
        // Select the mailbox
        session.select(mailbox)?;
        
        // Flag the message
        session.uid_store(email_id, "+FLAGS (\\Flagged)")?;
        
        Ok(())
    }
    
    /// Unflags an email.
    ///
    /// # Parameters
    /// - `mailbox`: The mailbox containing the email
    /// - `email_id`: The ID of the email to unflag
    ///
    /// # Returns
    /// A Result indicating success or failure
    #[allow(dead_code)]
    pub async fn unflag_email(&self, mailbox: &str, email_id: &str) -> Result<()> {
        let session_arc = self.session.as_ref()
            .ok_or_else(|| anyhow!("Not connected to IMAP server"))?;
            
        let mut session = session_arc.lock().await;
        
        // Select the mailbox
        session.select(mailbox)?;
        
        // Unflag the message
        session.uid_store(email_id, "-FLAGS (\\Flagged)")?;
        
        Ok(())
    }
    
    /// Deletes an email.
    ///
    /// # Parameters
    /// - `mailbox`: The mailbox containing the email
    /// - `email_id`: The ID of the email to delete
    ///
    /// # Returns
    /// A Result indicating success or failure
    #[allow(dead_code)]
    pub async fn delete_email(&self, mailbox: &str, email_id: &str) -> Result<()> {
        let session_arc = self.session.as_ref()
            .ok_or_else(|| anyhow!("Not connected to IMAP server"))?;
            
        let mut session = session_arc.lock().await;
        
        // Select the mailbox
        session.select(mailbox)?;
        
        // Mark the message for deletion
        session.uid_store(email_id, "+FLAGS (\\Deleted)")?;
        
        // Expunge the mailbox to remove deleted messages
        session.expunge()?;
        
        Ok(())
    }
    
    /// Checks if the client is connected.
    ///
    /// # Returns
    /// true if connected, false otherwise
    #[allow(dead_code)]
    pub async fn is_connected(&self) -> bool {
        let account = self.account.lock().await;
        account.imap_status == ConnectionStatus::Connected
    }
}
