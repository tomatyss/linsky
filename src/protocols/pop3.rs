//! POP3 protocol implementation for the Linksy email client.

use crate::config::ServerConfig;
use crate::models::{Account, ConnectionStatus, Email};
use anyhow::{anyhow, Result};
use log::{debug, error};
use std::sync::Arc;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
    sync::Mutex,
};
use native_tls::TlsConnector;
use tokio_native_tls::TlsStream;

/// Represents a POP3 client connection.
pub struct Pop3Client {
    /// The account this client is connected to
    account: Arc<Mutex<Account>>,
    /// The POP3 connection
    connection: Option<Arc<Mutex<Pop3Connection>>>,
}

/// Represents a POP3 connection.
enum Pop3Connection {
    /// Plain TCP connection
    Plain(BufReader<TcpStream>),
    /// TLS connection
    Tls(BufReader<TlsStream<TcpStream>>),
}

impl Pop3Client {
    /// Creates a new POP3 client for the specified account.
    ///
    /// # Parameters
    /// - `account`: The email account to connect to
    ///
    /// # Returns
    /// A new Pop3Client instance
    pub fn new(account: Arc<Mutex<Account>>) -> Self {
        Self {
            account,
            connection: None,
        }
    }
    
    /// Connects to the POP3 server.
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub async fn connect(&mut self) -> Result<()> {
        let account = self.account.lock().await;
        
        // Check if POP3 is configured
        let pop3_config = account.get_pop3_config()
            .ok_or_else(|| anyhow!("POP3 is not configured for this account"))?;
            
        // Clone the config for later use
        let pop3_config = pop3_config.clone();
        
        // Update connection status
        let mut account = account;
        account.pop3_status = ConnectionStatus::Connecting;
        drop(account); // Release the lock
        
        // Connect to the server
        let connection = self.create_connection(&pop3_config).await?;
        
        // Store the connection
        self.connection = Some(Arc::new(Mutex::new(connection)));
        
        // Update connection status
        let mut account = self.account.lock().await;
        account.pop3_status = ConnectionStatus::Connected;
        account.last_error = None;
        
        Ok(())
    }
    
    /// Creates a POP3 connection to the server.
    ///
    /// # Parameters
    /// - `config`: The server configuration
    ///
    /// # Returns
    /// A Result containing the POP3 connection or an error
    async fn create_connection(&self, config: &ServerConfig) -> Result<Pop3Connection> {
        // Connect to the server
        let addr = format!("{}:{}", config.host, config.port);
        let tcp_stream = TcpStream::connect(&addr).await?;
        
        let mut connection = if config.use_ssl {
            // Create TLS connector
            let connector = TlsConnector::new()?;
            let connector = tokio_native_tls::TlsConnector::from(connector);
            
            // Connect with TLS
            let tls_stream = connector.connect(&config.host, tcp_stream).await?;
            let reader = BufReader::new(tls_stream);
            Pop3Connection::Tls(reader)
        } else {
            // Use plain TCP
            let reader = BufReader::new(tcp_stream);
            Pop3Connection::Plain(reader)
        };
        
        // Read the greeting
        let greeting = self.read_response(&mut connection).await?;
        if !greeting.starts_with("+OK") {
            return Err(anyhow!("Invalid POP3 greeting: {}", greeting));
        }
        
        // Login to the server
        self.send_command(&mut connection, &format!("USER {}", config.username)).await?;
        let response = self.read_response(&mut connection).await?;
        if !response.starts_with("+OK") {
            return Err(anyhow!("USER command failed: {}", response));
        }
        
        self.send_command(&mut connection, &format!("PASS {}", config.password)).await?;
        let response = self.read_response(&mut connection).await?;
        if !response.starts_with("+OK") {
            return Err(anyhow!("PASS command failed: {}", response));
        }
        
        // Get message count
        self.send_command(&mut connection, "STAT").await?;
        let response = self.read_response(&mut connection).await?;
        if !response.starts_with("+OK") {
            return Err(anyhow!("STAT command failed: {}", response));
        }
        
        // Parse message count
        let parts: Vec<&str> = response.split_whitespace().collect();
        if parts.len() >= 3 {
            if let Ok(count) = parts[1].parse::<usize>() {
                // Update account with message count
                let mut account = self.account.lock().await;
                account.total_count = count;
                account.unread_count = count; // POP3 doesn't track read status
            }
        }
        
        Ok(connection)
    }
    
    /// Sends a command to the POP3 server.
    ///
    /// # Parameters
    /// - `connection`: The POP3 connection
    /// - `command`: The command to send
    ///
    /// # Returns
    /// A Result indicating success or failure
    async fn send_command(&self, connection: &mut Pop3Connection, command: &str) -> Result<()> {
        let command = format!("{}\r\n", command);
        
        match connection {
            Pop3Connection::Plain(reader) => {
                let stream = reader.get_mut();
                stream.write_all(command.as_bytes()).await?;
            },
            Pop3Connection::Tls(reader) => {
                let stream = reader.get_mut();
                stream.write_all(command.as_bytes()).await?;
            },
        }
        
        Ok(())
    }
    
    /// Reads a response from the POP3 server.
    ///
    /// # Parameters
    /// - `connection`: The POP3 connection
    ///
    /// # Returns
    /// A Result containing the response or an error
    async fn read_response(&self, connection: &mut Pop3Connection) -> Result<String> {
        let mut line = String::new();
        
        match connection {
            Pop3Connection::Plain(reader) => {
                reader.read_line(&mut line).await?;
            },
            Pop3Connection::Tls(reader) => {
                reader.read_line(&mut line).await?;
            },
        }
        
        // Remove trailing CRLF
        if line.ends_with("\r\n") {
            line.truncate(line.len() - 2);
        }
        
        Ok(line)
    }
    
    /// Reads a multi-line response from the POP3 server.
    ///
    /// # Parameters
    /// - `connection`: The POP3 connection
    ///
    /// # Returns
    /// A Result containing the response lines or an error
    async fn read_multiline_response(&self, connection: &mut Pop3Connection) -> Result<Vec<String>> {
        let mut lines = Vec::new();
        let mut line = String::new();
        
        match connection {
            Pop3Connection::Plain(reader) => {
                loop {
                    line.clear();
                    reader.read_line(&mut line).await?;
                    
                    // Remove trailing CRLF
                    if line.ends_with("\r\n") {
                        line.truncate(line.len() - 2);
                    }
                    
                    // Check for end of response
                    if line == "." {
                        break;
                    }
                    
                    // Handle byte-stuffing
                    if line.starts_with("..") {
                        line.remove(0);
                    }
                    
                    lines.push(line.clone());
                }
            },
            Pop3Connection::Tls(reader) => {
                loop {
                    line.clear();
                    reader.read_line(&mut line).await?;
                    
                    // Remove trailing CRLF
                    if line.ends_with("\r\n") {
                        line.truncate(line.len() - 2);
                    }
                    
                    // Check for end of response
                    if line == "." {
                        break;
                    }
                    
                    // Handle byte-stuffing
                    if line.starts_with("..") {
                        line.remove(0);
                    }
                    
                    lines.push(line.clone());
                }
            },
        }
        
        Ok(lines)
    }
    
    /// Disconnects from the POP3 server.
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub async fn disconnect(&mut self) -> Result<()> {
        if let Some(connection) = &self.connection {
            let mut connection = connection.lock().await;
            
            // Send QUIT command
            self.send_command(&mut connection, "QUIT").await?;
            let _ = self.read_response(&mut connection).await;
        }
        
        self.connection = None;
        
        let mut account = self.account.lock().await;
        account.pop3_status = ConnectionStatus::Disconnected;
        
        Ok(())
    }
    
    /// Fetches emails from the server.
    ///
    /// # Parameters
    /// - `limit`: Maximum number of emails to fetch
    ///
    /// # Returns
    /// A Result containing a vector of emails or an error
    pub async fn fetch_emails(&self, limit: usize) -> Result<Vec<Email>> {
        let connection_arc = self.connection.as_ref()
            .ok_or_else(|| anyhow!("Not connected to POP3 server"))?;
            
        let mut connection = connection_arc.lock().await;
        
        // Get message count
        self.send_command(&mut connection, "STAT").await?;
        let response = self.read_response(&mut connection).await?;
        if !response.starts_with("+OK") {
            return Err(anyhow!("STAT command failed: {}", response));
        }
        
        // Parse message count
        let parts: Vec<&str> = response.split_whitespace().collect();
        if parts.len() < 3 {
            return Err(anyhow!("Invalid STAT response: {}", response));
        }
        
        let count = parts[1].parse::<usize>()?;
        debug!("POP3 server has {} messages", count);
        
        // Update account with message count
        let mut account = self.account.lock().await;
        account.total_count = count;
        account.unread_count = count; // POP3 doesn't track read status
        let account_id = account.config.id.clone();
        drop(account); // Release the lock
        
        // Determine which messages to fetch
        let start = if count > limit {
            count - limit
        } else {
            0
        };
        
        let mut emails = Vec::new();
        
        // Fetch messages
        for i in start..count {
            let msg_num = i + 1; // POP3 message numbers are 1-based
            
            // Retrieve the message
            self.send_command(&mut connection, &format!("RETR {}", msg_num)).await?;
            let response = self.read_response(&mut connection).await?;
            if !response.starts_with("+OK") {
                error!("RETR command failed for message {}: {}", msg_num, response);
                continue;
            }
            
            // Read the message
            let lines = self.read_multiline_response(&mut connection).await?;
            let raw_data = lines.join("\r\n").into_bytes();
            
            // Parse the email
            match Email::parse_from_raw(&raw_data, &account_id, "INBOX") {
                Ok(email) => {
                    emails.push(email);
                },
                Err(e) => {
                    error!("Failed to parse email: {}", e);
                }
            }
        }
        
        // Sort emails by date (newest first)
        emails.sort_by(|a, b| b.date.cmp(&a.date));
        
        Ok(emails)
    }
    
    /// Deletes an email from the server.
    ///
    /// # Parameters
    /// - `message_number`: The message number to delete
    ///
    /// # Returns
    /// A Result indicating success or failure
    #[allow(dead_code)]
    pub async fn delete_email(&self, message_number: usize) -> Result<()> {
        let connection_arc = self.connection.as_ref()
            .ok_or_else(|| anyhow!("Not connected to POP3 server"))?;
            
        let mut connection = connection_arc.lock().await;
        
        // Delete the message
        self.send_command(&mut connection, &format!("DELE {}", message_number)).await?;
        let response = self.read_response(&mut connection).await?;
        if !response.starts_with("+OK") {
            return Err(anyhow!("DELE command failed: {}", response));
        }
        
        Ok(())
    }
    
    /// Checks if the client is connected.
    ///
    /// # Returns
    /// true if connected, false otherwise
    #[allow(dead_code)]
    pub async fn is_connected(&self) -> bool {
        let account = self.account.lock().await;
        account.pop3_status == ConnectionStatus::Connected
    }
}
