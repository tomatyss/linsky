//! Storage implementation for the Linksy email client.
//! 
//! This module handles local storage and caching of emails and other data.

use crate::models::{Email, Account};
use anyhow::Result;
use sled::Db;
use std::path::Path;

/// Represents the email storage.
pub struct EmailStorage {
    /// The database instance
    db: Db,
}

impl EmailStorage {
    /// Creates a new EmailStorage instance.
    ///
    /// # Parameters
    /// - `path`: Path to the storage directory
    ///
    /// # Returns
    /// A Result containing the EmailStorage or an error
    pub fn new(path: &Path) -> Result<Self> {
        // Create the database
        let db = sled::open(path)?;
        
        Ok(Self { db })
    }
    
    /// Stores an email in the database.
    ///
    /// # Parameters
    /// - `email`: The email to store
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub fn store_email(&self, email: &Email) -> Result<()> {
        // Create a key for the email
        let key = format!("email:{}:{}:{}", email.account_id, email.folder, email.id);
        
        // Serialize the email
        let value = serde_json::to_vec(email)?;
        
        // Store the email
        self.db.insert(key.as_bytes(), value)?;
        self.db.flush()?;
        
        Ok(())
    }
    
    /// Retrieves an email from the database.
    ///
    /// # Parameters
    /// - `account_id`: The account ID
    /// - `folder`: The folder/mailbox
    /// - `email_id`: The email ID
    ///
    /// # Returns
    /// A Result containing the Email or an error
    #[allow(dead_code)]
    pub fn get_email(&self, account_id: &str, folder: &str, email_id: &str) -> Result<Option<Email>> {
        // Create a key for the email
        let key = format!("email:{}:{}:{}", account_id, folder, email_id);
        
        // Retrieve the email
        if let Some(value) = self.db.get(key.as_bytes())? {
            // Deserialize the email
            let email: Email = serde_json::from_slice(&value)?;
            Ok(Some(email))
        } else {
            Ok(None)
        }
    }
    
    /// Retrieves all emails for an account and folder.
    ///
    /// # Parameters
    /// - `account_id`: The account ID
    /// - `folder`: The folder/mailbox
    ///
    /// # Returns
    /// A Result containing a vector of emails or an error
    pub fn get_emails(&self, account_id: &str, folder: &str) -> Result<Vec<Email>> {
        // Create a prefix for the emails
        let prefix = format!("email:{}:{}:", account_id, folder);
        
        // Retrieve all emails with the prefix
        let mut emails = Vec::new();
        
        for result in self.db.scan_prefix(prefix.as_bytes()) {
            let (_, value) = result?;
            let email: Email = serde_json::from_slice(&value)?;
            emails.push(email);
        }
        
        // Sort emails by date (newest first)
        emails.sort_by(|a, b| b.date.cmp(&a.date));
        
        Ok(emails)
    }
    
    /// Deletes an email from the database.
    ///
    /// # Parameters
    /// - `account_id`: The account ID
    /// - `folder`: The folder/mailbox
    /// - `email_id`: The email ID
    ///
    /// # Returns
    /// A Result indicating success or failure
    #[allow(dead_code)]
    pub fn delete_email(&self, account_id: &str, folder: &str, email_id: &str) -> Result<()> {
        // Create a key for the email
        let key = format!("email:{}:{}:{}", account_id, folder, email_id);
        
        // Delete the email
        self.db.remove(key.as_bytes())?;
        self.db.flush()?;
        
        Ok(())
    }
    
    /// Updates an email in the database.
    ///
    /// # Parameters
    /// - `email`: The email to update
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub fn update_email(&self, email: &Email) -> Result<()> {
        // Store the email (overwrites existing)
        self.store_email(email)
    }
    
    /// Stores account information in the database.
    ///
    /// # Parameters
    /// - `account`: The account to store
    ///
    /// # Returns
    /// A Result indicating success or failure
    #[allow(dead_code)]
    pub fn store_account(&self, account: &Account) -> Result<()> {
        // Create a key for the account
        let key = format!("account:{}", account.config.id);
        
        // Serialize the account
        let value = serde_json::to_vec(account)?;
        
        // Store the account
        self.db.insert(key.as_bytes(), value)?;
        self.db.flush()?;
        
        Ok(())
    }
    
    /// Retrieves account information from the database.
    ///
    /// # Parameters
    /// - `account_id`: The account ID
    ///
    /// # Returns
    /// A Result containing the Account or an error
    #[allow(dead_code)]
    pub fn get_account(&self, account_id: &str) -> Result<Option<Account>> {
        // Create a key for the account
        let key = format!("account:{}", account_id);
        
        // Retrieve the account
        if let Some(value) = self.db.get(key.as_bytes())? {
            // Deserialize the account
            let account: Account = serde_json::from_slice(&value)?;
            Ok(Some(account))
        } else {
            Ok(None)
        }
    }
    
    /// Retrieves all accounts from the database.
    ///
    /// # Returns
    /// A Result containing a vector of accounts or an error
    #[allow(dead_code)]
    pub fn get_all_accounts(&self) -> Result<Vec<Account>> {
        // Create a prefix for the accounts
        let prefix = "account:";
        
        // Retrieve all accounts with the prefix
        let mut accounts = Vec::new();
        
        for result in self.db.scan_prefix(prefix.as_bytes()) {
            let (_, value) = result?;
            let account: Account = serde_json::from_slice(&value)?;
            accounts.push(account);
        }
        
        Ok(accounts)
    }
    
    /// Deletes an account from the database.
    ///
    /// # Parameters
    /// - `account_id`: The account ID
    ///
    /// # Returns
    /// A Result indicating success or failure
    #[allow(dead_code)]
    pub fn delete_account(&self, account_id: &str) -> Result<()> {
        // Create a key for the account
        let key = format!("account:{}", account_id);
        
        // Delete the account
        self.db.remove(key.as_bytes())?;
        
        // Delete all emails for the account
        let email_prefix = format!("email:{}:", account_id);
        for result in self.db.scan_prefix(email_prefix.as_bytes()) {
            let (key, _) = result?;
            self.db.remove(key)?;
        }
        
        self.db.flush()?;
        
        Ok(())
    }
    
    /// Closes the database.
    ///
    /// # Returns
    /// A Result indicating success or failure
    #[allow(dead_code)]
    pub fn close(&self) -> Result<()> {
        self.db.flush()?;
        Ok(())
    }
}
