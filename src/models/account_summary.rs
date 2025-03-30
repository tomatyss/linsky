//! Account summary model for the Linksy email client.
//!
//! This module provides a lightweight representation of account information
//! that can be used for rendering without requiring mutex locks.

use crate::models::ConnectionStatus;
use serde::{Deserialize, Serialize};

/// Represents a lightweight summary of an email account for rendering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountSummary {
    /// Account ID
    pub id: String,
    /// Account name
    pub name: String,
    /// Email address
    pub email: String,
    /// IMAP connection status
    pub imap_status: ConnectionStatus,
    /// SMTP connection status
    pub smtp_status: ConnectionStatus,
    /// POP3 connection status
    pub pop3_status: ConnectionStatus,
    /// Number of unread messages
    pub unread_count: usize,
    /// Total number of messages
    pub total_count: usize,
}

impl AccountSummary {
    /// Gets the display name for the account.
    ///
    /// # Returns
    /// A string containing the display name
    pub fn get_display_name(&self) -> String {
        format!("{} <{}>", self.name, self.email)
    }
    
    /// Gets the connection status summary.
    ///
    /// # Returns
    /// A string describing the connection status
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
    pub fn get_unread_summary(&self) -> String {
        if self.unread_count > 0 {
            format!("{} unread", self.unread_count)
        } else {
            "No unread messages".to_string()
        }
    }
}
