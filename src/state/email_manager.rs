//! Email management for the Linksy email client.

use crate::models::{Account, ConnectionStatus, Email};
use crate::protocols::{ImapClient, Pop3Client};
use crate::storage::EmailStorage;
use anyhow::Result;
use log::error;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Manages email operations.
pub struct EmailManager {
    /// Email storage
    storage: EmailStorage,
}

impl EmailManager {
    /// Creates a new EmailManager instance.
    ///
    /// # Parameters
    /// - `storage`: The email storage
    ///
    /// # Returns
    /// A new EmailManager instance
    pub fn new(storage: EmailStorage) -> Self {
        Self {
            storage,
        }
    }
    
    /// Loads emails for the specified account and folder.
    ///
    /// # Parameters
    /// - `account`: The account
    /// - `imap_client`: The IMAP client
    /// - `pop3_client`: The POP3 client
    /// - `folder`: The folder/mailbox
    /// - `limit`: Maximum number of emails to fetch
    ///
    /// # Returns
    /// A Result containing a vector of emails
    pub async fn load_emails(
        &self,
        account: &Arc<Mutex<Account>>,
        imap_client: Option<&Arc<Mutex<ImapClient>>>,
        pop3_client: Option<&Arc<Mutex<Pop3Client>>>,
        folder: &str,
        limit: usize,
    ) -> Result<Vec<Email>> {
        // Get account ID and check connection status
        let account_id;
        let has_imap;
        let has_pop3;
        let imap_status;
        let pop3_status;
        
        {
            let account_lock = account.lock().await;
            account_id = account_lock.config.id.clone();
            has_imap = account_lock.has_imap();
            has_pop3 = account_lock.has_pop3();
            imap_status = account_lock.imap_status;
            pop3_status = account_lock.pop3_status;
        }
        
        // Try to load emails from storage first
        let mut emails = match self.storage.get_emails(&account_id, folder) {
            Ok(emails) => emails,
            Err(e) => {
                error!("Failed to load emails from storage: {}", e);
                Vec::new()
            }
        };
        
        // Check if account has IMAP and is connected
        if has_imap && imap_status == ConnectionStatus::Connected && imap_client.is_some() {
            // Fetch emails from IMAP
            if let Some(imap_client) = imap_client {
                let client = imap_client.lock().await;
                
                match client.fetch_emails(folder, limit).await {
                    Ok(fetched_emails) => {
                        // Store emails in storage
                        for email in &fetched_emails {
                            if let Err(e) = self.storage.store_email(email) {
                                error!("Failed to store email: {}", e);
                            }
                        }
                        
                        emails = fetched_emails;
                    },
                    Err(e) => {
                        error!("Failed to fetch emails: {}", e);
                    }
                }
            }
        } else if has_pop3 && pop3_status == ConnectionStatus::Connected && pop3_client.is_some() {
            // Fetch emails from POP3
            if let Some(pop3_client) = pop3_client {
                let client = pop3_client.lock().await;
                
                match client.fetch_emails(limit).await {
                    Ok(fetched_emails) => {
                        // Store emails in storage
                        for email in &fetched_emails {
                            if let Err(e) = self.storage.store_email(email) {
                                error!("Failed to store email: {}", e);
                            }
                        }
                        
                        emails = fetched_emails;
                    },
                    Err(e) => {
                        error!("Failed to fetch emails: {}", e);
                    }
                }
            }
        }
        
        Ok(emails)
    }
    
    /// Marks an email as read.
    ///
    /// # Parameters
    /// - `imap_client`: The IMAP client
    /// - `email`: The email to mark
    /// - `folder`: The folder/mailbox
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub async fn mark_as_read(
        &self,
        imap_client: &Arc<Mutex<ImapClient>>,
        email: &Email,
        folder: &str,
    ) -> Result<()> {
        // Mark as read using IMAP
        let client = imap_client.lock().await;
        
        if let Err(e) = client.mark_as_read(folder, &email.id).await {
            error!("Failed to mark email as read: {}", e);
            return Err(e.into());
        }
        
        // Update email in storage
        let mut updated_email = email.clone();
        updated_email.is_read = true;
        if let Err(e) = self.storage.update_email(&updated_email) {
            error!("Failed to update email in storage: {}", e);
            return Err(e);
        }
        
        Ok(())
    }
    
    /// Marks an email as unread.
    ///
    /// # Parameters
    /// - `imap_client`: The IMAP client
    /// - `email`: The email to mark
    /// - `folder`: The folder/mailbox
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub async fn mark_as_unread(
        &self,
        imap_client: &Arc<Mutex<ImapClient>>,
        email: &Email,
        folder: &str,
    ) -> Result<()> {
        // Mark as unread using IMAP
        let client = imap_client.lock().await;
        
        if let Err(e) = client.mark_as_unread(folder, &email.id).await {
            error!("Failed to mark email as unread: {}", e);
            return Err(e.into());
        }
        
        // Update email in storage
        let mut updated_email = email.clone();
        updated_email.is_read = false;
        if let Err(e) = self.storage.update_email(&updated_email) {
            error!("Failed to update email in storage: {}", e);
            return Err(e);
        }
        
        Ok(())
    }
    
    /// Flags an email.
    ///
    /// # Parameters
    /// - `imap_client`: The IMAP client
    /// - `email`: The email to flag
    /// - `folder`: The folder/mailbox
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub async fn flag_email(
        &self,
        imap_client: &Arc<Mutex<ImapClient>>,
        email: &Email,
        folder: &str,
    ) -> Result<()> {
        // Flag the email using IMAP
        let client = imap_client.lock().await;
        
        if let Err(e) = client.flag_email(folder, &email.id).await {
            error!("Failed to flag email: {}", e);
            return Err(e.into());
        }
        
        // Update email in storage
        let mut updated_email = email.clone();
        updated_email.is_flagged = true;
        if let Err(e) = self.storage.update_email(&updated_email) {
            error!("Failed to update email in storage: {}", e);
            return Err(e);
        }
        
        Ok(())
    }
    
    /// Unflags an email.
    ///
    /// # Parameters
    /// - `imap_client`: The IMAP client
    /// - `email`: The email to unflag
    /// - `folder`: The folder/mailbox
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub async fn unflag_email(
        &self,
        imap_client: &Arc<Mutex<ImapClient>>,
        email: &Email,
        folder: &str,
    ) -> Result<()> {
        // Unflag the email using IMAP
        let client = imap_client.lock().await;
        
        if let Err(e) = client.unflag_email(folder, &email.id).await {
            error!("Failed to unflag email: {}", e);
            return Err(e.into());
        }
        
        // Update email in storage
        let mut updated_email = email.clone();
        updated_email.is_flagged = false;
        if let Err(e) = self.storage.update_email(&updated_email) {
            error!("Failed to update email in storage: {}", e);
            return Err(e);
        }
        
        Ok(())
    }
    
    /// Deletes an email.
    ///
    /// # Parameters
    /// - `imap_client`: The IMAP client
    /// - `email`: The email to delete
    /// - `folder`: The folder/mailbox
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub async fn delete_email(
        &self,
        imap_client: &Arc<Mutex<ImapClient>>,
        email: &Email,
        folder: &str,
    ) -> Result<()> {
        // Delete the email using IMAP
        let client = imap_client.lock().await;
        
        if let Err(e) = client.delete_email(folder, &email.id).await {
            error!("Failed to delete email: {}", e);
            return Err(e.into());
        }
        
        // Delete email from storage
        if let Err(e) = self.storage.delete_email(&email.account_id, folder, &email.id) {
            error!("Failed to delete email from storage: {}", e);
            return Err(e);
        }
        
        Ok(())
    }
    
    /// Gets an email from storage.
    ///
    /// # Parameters
    /// - `account_id`: The account ID
    /// - `folder`: The folder/mailbox
    /// - `email_id`: The email ID
    ///
    /// # Returns
    /// A Result containing an Option with the email
    pub fn get_email(&self, account_id: &str, folder: &str, email_id: &str) -> Result<Option<Email>> {
        self.storage.get_email(account_id, folder, email_id)
    }
    
    /// Updates an email in storage.
    ///
    /// # Parameters
    /// - `email`: The email to update
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub fn update_email(&self, email: &Email) -> Result<()> {
        self.storage.update_email(email)
    }
}
