//! Email account model for the Linksy email client.

use crate::config::{EmailAccount, ServerConfig};
use serde::{Deserialize, Serialize};

/// Represents the status of an email account connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionStatus {
    /// Account is disconnected
    Disconnected,
    /// Account is connecting
    Connecting,
    /// Account is connected
    Connected,
    /// Account connection failed
    Failed,
}

/// Represents an email account with connection status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    /// Account configuration
    pub config: EmailAccount,
    /// IMAP connection status
    pub imap_status: ConnectionStatus,
    /// SMTP connection status
    pub smtp_status: ConnectionStatus,
    /// POP3 connection status
    pub pop3_status: ConnectionStatus,
    /// Last error message (if any)
    pub last_error: Option<String>,
    /// Number of unread messages
    pub unread_count: usize,
    /// Total number of messages
    pub total_count: usize,
    /// Available folders/mailboxes
    pub folders: Vec<String>,
}

impl Account {
    /// Creates a new Account from an EmailAccount configuration.
    ///
    /// # Parameters
    /// - `config`: The email account configuration
    ///
    /// # Returns
    /// A new Account instance
    pub fn new(config: EmailAccount) -> Self {
        Self {
            config,
            imap_status: ConnectionStatus::Disconnected,
            smtp_status: ConnectionStatus::Disconnected,
            pop3_status: ConnectionStatus::Disconnected,
            last_error: None,
            unread_count: 0,
            total_count: 0,
            folders: vec!["INBOX".to_string()],
        }
    }
    
    /// Gets the display name for the account.
    ///
    /// # Returns
    /// A string containing the display name
    pub fn get_display_name(&self) -> String {
        format!("{} <{}>", self.config.name, self.config.email)
    }
    
    /// Gets the connection status summary.
    ///
    /// # Returns
    /// A string describing the connection status
    #[allow(dead_code)]
    pub fn get_status_summary(&self) -> String {
        let imap_status = match self.imap_status {
            ConnectionStatus::Connected => "✓",
            ConnectionStatus::Connecting => "⟳",
            ConnectionStatus::Failed => "✗",
            ConnectionStatus::Disconnected => "-",
        };
        
        let smtp_status = match self.smtp_status {
            ConnectionStatus::Connected => "✓",
            ConnectionStatus::Connecting => "⟳",
            ConnectionStatus::Failed => "✗",
            ConnectionStatus::Disconnected => "-",
        };
        
        let pop3_status = match self.pop3_status {
            ConnectionStatus::Connected => "✓",
            ConnectionStatus::Connecting => "⟳",
            ConnectionStatus::Failed => "✗",
            ConnectionStatus::Disconnected => "-",
        };
        
        format!("IMAP: {} | SMTP: {} | POP3: {}", imap_status, smtp_status, pop3_status)
    }
    
    /// Gets the unread message count as a string.
    ///
    /// # Returns
    /// A string describing the unread message count
    #[allow(dead_code)]
    pub fn get_unread_summary(&self) -> String {
        if self.unread_count > 0 {
            format!("{} unread", self.unread_count)
        } else {
            "No unread messages".to_string()
        }
    }
    
    /// Checks if the account has IMAP configured.
    ///
    /// # Returns
    /// true if IMAP is configured, false otherwise
    pub fn has_imap(&self) -> bool {
        self.config.imap.is_some()
    }
    
    /// Checks if the account has POP3 configured.
    ///
    /// # Returns
    /// true if POP3 is configured, false otherwise
    pub fn has_pop3(&self) -> bool {
        self.config.pop3.is_some()
    }
    
    /// Gets the IMAP server configuration.
    ///
    /// # Returns
    /// An Option containing the IMAP server configuration
    pub fn get_imap_config(&self) -> Option<&ServerConfig> {
        self.config.imap.as_ref()
    }
    
    /// Gets the POP3 server configuration.
    ///
    /// # Returns
    /// An Option containing the POP3 server configuration
    pub fn get_pop3_config(&self) -> Option<&ServerConfig> {
        self.config.pop3.as_ref()
    }
    
    /// Gets the SMTP server configuration.
    ///
    /// # Returns
    /// A reference to the SMTP server configuration
    pub fn get_smtp_config(&self) -> &ServerConfig {
        &self.config.smtp
    }
}
