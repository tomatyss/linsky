//! SMTP protocol implementation for the Linksy email client.

use crate::config::ServerConfig;
use crate::models::{Account, ConnectionStatus, Email};
use anyhow::{anyhow, Result};
use lettre::{
    message::{header, MultiPart, SinglePart},
    transport::smtp::{authentication::Credentials, client::TlsParameters},
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Represents an SMTP client for sending emails.
pub struct SmtpClient {
    /// The account this client is connected to
    account: Arc<Mutex<Account>>,
    /// The SMTP transport
    transport: Option<AsyncSmtpTransport<Tokio1Executor>>,
}

impl SmtpClient {
    /// Creates a new SMTP client for the specified account.
    ///
    /// # Parameters
    /// - `account`: The email account to connect to
    ///
    /// # Returns
    /// A new SmtpClient instance
    pub fn new(account: Arc<Mutex<Account>>) -> Self {
        Self {
            account,
            transport: None,
        }
    }
    
    /// Connects to the SMTP server.
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub async fn connect(&mut self) -> Result<()> {
        let mut account = self.account.lock().await;
        
        // Get SMTP configuration
        let smtp_config = account.get_smtp_config().clone();
        
        // Update connection status
        account.smtp_status = ConnectionStatus::Connecting;
        drop(account); // Release the lock
        
        // Create SMTP transport
        let transport = self.create_transport(&smtp_config)?;
        
        // Test the connection
        transport.test_connection().await?;
        
        // Store the transport
        self.transport = Some(transport);
        
        // Update connection status
        let mut account = self.account.lock().await;
        account.smtp_status = ConnectionStatus::Connected;
        account.last_error = None;
        
        Ok(())
    }
    
    /// Creates an SMTP transport.
    ///
    /// # Parameters
    /// - `config`: The server configuration
    ///
    /// # Returns
    /// A Result containing the SMTP transport or an error
    fn create_transport(&self, config: &ServerConfig) -> Result<AsyncSmtpTransport<Tokio1Executor>> {
        // Create credentials
        let creds = Credentials::new(config.username.clone(), config.password.clone());
        
        // Create TLS parameters
        let tls_parameters = TlsParameters::new(config.host.clone())?;
        
        // Create transport builder
        let mut builder = AsyncSmtpTransport::<Tokio1Executor>::relay(&config.host)?
            .port(config.port)
            .credentials(creds);
            
        // Configure TLS if needed
        builder = if config.use_ssl {
            builder.tls(lettre::transport::smtp::client::Tls::Required(tls_parameters))
        } else {
            builder.tls(lettre::transport::smtp::client::Tls::None)
        };
        
        Ok(builder.build())
    }
    
    /// Disconnects from the SMTP server.
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub async fn disconnect(&mut self) -> Result<()> {
        self.transport = None;
        
        let mut account = self.account.lock().await;
        account.smtp_status = ConnectionStatus::Disconnected;
        
        Ok(())
    }
    
    /// Sends an email.
    ///
    /// # Parameters
    /// - `email`: The email to send
    ///
    /// # Returns
    /// A Result indicating success or failure
    #[allow(dead_code)]
    pub async fn send_email(&self, email: &Email) -> Result<()> {
        let transport = self.transport.as_ref()
            .ok_or_else(|| anyhow!("Not connected to SMTP server"))?;
            
        // Get account information
        let account = self.account.lock().await;
        let from_email = &account.config.email;
        let from_name = &account.config.name;
        
        // Create email builder
        let mut builder = Message::builder()
            .from(format!("{} <{}>", from_name, from_email).parse()?)
            .subject(&email.subject);
            
        // Add recipients
        for to in &email.to {
            builder = builder.to(to.parse()?);
        }
        
        for cc in &email.cc {
            builder = builder.cc(cc.parse()?);
        }
        
        for bcc in &email.bcc {
            builder = builder.bcc(bcc.parse()?);
        }
        
        // Create email body
        let message = if !email.attachments.is_empty() {
            // For emails with attachments
            if let (Some(text), Some(html)) = (&email.body_text, &email.body_html) {
                // Both text and HTML with attachments
                let text_part = SinglePart::builder()
                    .header(header::ContentType::TEXT_PLAIN)
                    .body(text.clone());
                
                let html_part = SinglePart::builder()
                    .header(header::ContentType::TEXT_HTML)
                    .body(html.clone());
                
                // Create alternative part for text and HTML
                let alternative = MultiPart::alternative()
                    .singlepart(text_part)
                    .singlepart(html_part);
                
                // Create mixed part for content and attachments
                let mut mixed = MultiPart::mixed().multipart(alternative);
                
                // Add attachments
                for attachment in &email.attachments {
                    let attachment_part = SinglePart::builder()
                        .header(header::ContentType::parse(&attachment.content_type)?)
                        .header(header::ContentDisposition::attachment(&attachment.filename))
                        .body(attachment.data.clone());
                    
                    mixed = mixed.singlepart(attachment_part);
                }
                
                builder.multipart(mixed)?
            } else if let Some(text) = &email.body_text {
                // Text only with attachments
                let text_part = SinglePart::builder()
                    .header(header::ContentType::TEXT_PLAIN)
                    .body(text.clone());
                
                // Create mixed part for content and attachments
                let mut mixed = MultiPart::mixed().singlepart(text_part);
                
                // Add attachments
                for attachment in &email.attachments {
                    let attachment_part = SinglePart::builder()
                        .header(header::ContentType::parse(&attachment.content_type)?)
                        .header(header::ContentDisposition::attachment(&attachment.filename))
                        .body(attachment.data.clone());
                    
                    mixed = mixed.singlepart(attachment_part);
                }
                
                builder.multipart(mixed)?
            } else if let Some(html) = &email.body_html {
                // HTML only with attachments
                let html_part = SinglePart::builder()
                    .header(header::ContentType::TEXT_HTML)
                    .body(html.clone());
                
                // Create mixed part for content and attachments
                let mut mixed = MultiPart::mixed().singlepart(html_part);
                
                // Add attachments
                for attachment in &email.attachments {
                    let attachment_part = SinglePart::builder()
                        .header(header::ContentType::parse(&attachment.content_type)?)
                        .header(header::ContentDisposition::attachment(&attachment.filename))
                        .body(attachment.data.clone());
                    
                    mixed = mixed.singlepart(attachment_part);
                }
                
                builder.multipart(mixed)?
            } else {
                // Attachments only
                // Start with a single attachment
                let first_attachment = &email.attachments[0];
                let first_part = SinglePart::builder()
                    .header(header::ContentType::parse(&first_attachment.content_type)?)
                    .header(header::ContentDisposition::attachment(&first_attachment.filename))
                    .body(first_attachment.data.clone());
                
                // Create mixed part with the first attachment
                let mut mixed = MultiPart::mixed().singlepart(first_part);
                
                // Add remaining attachments
                for attachment in &email.attachments[1..] {
                    let attachment_part = SinglePart::builder()
                        .header(header::ContentType::parse(&attachment.content_type)?)
                        .header(header::ContentDisposition::attachment(&attachment.filename))
                        .body(attachment.data.clone());
                    
                    mixed = mixed.singlepart(attachment_part);
                }
                
                builder.multipart(mixed)?
            }
        } else if let (Some(text), Some(html)) = (&email.body_text, &email.body_html) {
            // Both text and HTML without attachments
            let text_part = SinglePart::builder()
                .header(header::ContentType::TEXT_PLAIN)
                .body(text.clone());
            
            let html_part = SinglePart::builder()
                .header(header::ContentType::TEXT_HTML)
                .body(html.clone());
            
            // Create alternative part for text and HTML
            let alternative = MultiPart::alternative()
                .singlepart(text_part)
                .singlepart(html_part);
            
            builder.multipart(alternative)?
        } else if let Some(html) = &email.body_html {
            // HTML only
            builder.header(header::ContentType::TEXT_HTML)
                .body(html.clone())?
        } else if let Some(text) = &email.body_text {
            // Plain text email
            builder.header(header::ContentType::TEXT_PLAIN)
                .body(text.clone())?
        } else {
            // Empty email
            builder.body("".to_string())?
        };
        
        // Send the email
        transport.send(message).await?;
        
        Ok(())
    }
    
    /// Creates a new email with default values for the account.
    ///
    /// # Returns
    /// A new Email instance with default values
    #[allow(dead_code)]
    pub async fn create_new_email(&self) -> Email {
        let account = self.account.lock().await;
        let account_id = account.config.id.clone();
        let from = account.config.email.clone();
        let from_name = Some(account.config.name.clone());
        
        let mut email = Email::new();
        email.account_id = account_id;
        email.from = from;
        email.from_name = from_name;
        
        email
    }
    
    /// Checks if the client is connected.
    ///
    /// # Returns
    /// true if connected, false otherwise
    #[allow(dead_code)]
    pub async fn is_connected(&self) -> bool {
        let account = self.account.lock().await;
        account.smtp_status == ConnectionStatus::Connected
    }
}
