//! Email message model for the Linksy email client.

use crate::models::attachment::Attachment;
use mail_parser::{MessageParser, MimeHeaders};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Represents an email message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Email {
    /// Unique identifier for the email
    pub id: String,
    /// Email subject
    pub subject: String,
    /// Sender's email address
    pub from: String,
    /// Sender's display name (if available)
    pub from_name: Option<String>,
    /// List of recipient email addresses
    pub to: Vec<String>,
    /// List of CC recipient email addresses
    pub cc: Vec<String>,
    /// List of BCC recipient email addresses
    pub bcc: Vec<String>,
    /// Email body in plain text format
    pub body_text: Option<String>,
    /// Email body in HTML format
    pub body_html: Option<String>,
    /// Date when the email was received
    pub date: SystemTime,
    /// List of attachments
    pub attachments: Vec<Attachment>,
    /// Whether the email has been read
    pub is_read: bool,
    /// Whether the email has been flagged
    pub is_flagged: bool,
    /// Email headers
    pub headers: Vec<(String, String)>,
    /// Account ID this email belongs to
    pub account_id: String,
    /// Folder/mailbox this email belongs to
    pub folder: String,
}

impl Email {
    /// Creates a new empty email.
    ///
    /// # Returns
    /// A new Email instance with default values
    pub fn new() -> Self {
        Self {
            id: String::new(),
            subject: String::new(),
            from: String::new(),
            from_name: None,
            to: Vec::new(),
            cc: Vec::new(),
            bcc: Vec::new(),
            body_text: None,
            body_html: None,
            date: SystemTime::now(),
            attachments: Vec::new(),
            is_read: false,
            is_flagged: false,
            headers: Vec::new(),
            account_id: String::new(),
            folder: "INBOX".to_string(),
        }
    }
    
    /// Parses an email from raw message data.
    ///
    /// # Parameters
    /// - `raw_data`: Raw email message data
    /// - `account_id`: ID of the account this email belongs to
    /// - `folder`: Folder/mailbox this email belongs to
    ///
    /// # Returns
    /// A Result containing the parsed Email or an error
    pub fn parse_from_raw(raw_data: &[u8], account_id: &str, folder: &str) -> anyhow::Result<Self> {
        let message = MessageParser::default().parse(raw_data)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse email message"))?;
        
        let mut email = Email::new();
        
        // Set basic properties
        email.id = message.message_id().unwrap_or_default().to_string();
        email.subject = message.subject().unwrap_or_default().to_string();
        email.account_id = account_id.to_string();
        email.folder = folder.to_string();
        
        // Set sender information
        if let Some(from) = message.from() {
            if let Some(addr) = from.first() {
                email.from = addr.address().unwrap_or_default().to_string();
                email.from_name = addr.name().map(|s| s.to_string());
            }
        }
        
        // Set recipients
        if let Some(to) = message.to() {
            email.to = to.iter()
                .filter_map(|addr| addr.address().map(|s| s.to_string()))
                .collect();
        }
        
        if let Some(cc) = message.cc() {
            email.cc = cc.iter()
                .filter_map(|addr| addr.address().map(|s| s.to_string()))
                .collect();
        }
        
        if let Some(bcc) = message.bcc() {
            email.bcc = bcc.iter()
                .filter_map(|addr| addr.address().map(|s| s.to_string()))
                .collect();
        }
        
        // Set date
        if let Some(date) = message.date() {
            email.date = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(date.to_timestamp() as u64);
        }
        
        // Set body content
        let text_bodies: Vec<_> = message.text_bodies().collect();
        if !text_bodies.is_empty() {
            email.body_text = Some(String::from_utf8_lossy(text_bodies[0].contents()).to_string());
        }
        
        let html_bodies: Vec<_> = message.html_bodies().collect();
        if !html_bodies.is_empty() {
            email.body_html = Some(String::from_utf8_lossy(html_bodies[0].contents()).to_string());
        }
        
        // Set attachments
        for attachment in message.attachments() {
            if let Some(filename) = attachment.attachment_name() {
                let content_type = attachment.content_type()
                    .map(|ct| {
                        let ctype = ct.ctype();
                        let subtype = ct.subtype().unwrap_or("octet-stream");
                        format!("{}/{}", ctype, subtype)
                    })
                    .unwrap_or_else(|| "application/octet-stream".to_string());
                
                let body_bytes = match &attachment.body {
                    mail_parser::PartType::Text(text) => text.as_bytes(),
                    mail_parser::PartType::Binary(binary) => binary,
                    _ => &[]
                };
                
                let new_attachment = Attachment {
                    id: uuid::Uuid::new_v4().to_string(),
                    filename: filename.to_string(),
                    content_type,
                    size: body_bytes.len(),
                    data: body_bytes.to_vec(),
                };
                email.attachments.push(new_attachment);
            }
        }
        
        // Set headers
        for header in message.headers() {
            let name = header.name();
            let value = header.value();
            email.headers.push((name.to_string(), format!("{:?}", value)));
        }
        
        Ok(email)
    }
    
    /// Gets a summary of the email for display in lists.
    ///
    /// # Returns
    /// A string containing a summary of the email
    pub fn get_summary(&self) -> String {
        let from_display = if let Some(name) = &self.from_name {
            format!("{} <{}>", name, self.from)
        } else {
            self.from.clone()
        };
        
        let flag = if self.is_flagged { "🚩 " } else { "" };
        let read = if self.is_read { "" } else { "📩 " };
        
        format!("{}{}{} - {}", flag, read, from_display, self.subject)
    }
}

impl Default for Email {
    fn default() -> Self {
        Self::new()
    }
}
