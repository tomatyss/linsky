//! Main application logic for the Linksy email client UI.

use crate::config::ConfigManager;
use crate::models::{Account, ConnectionStatus, Email};
use crate::protocols::{ImapClient, Pop3Client, SmtpClient};
use crate::storage::EmailStorage;
use crate::ui::{init_terminal, restore_terminal, wait_for_key, is_key_with_modifier};
use anyhow::{anyhow, Result};
use crossterm::event::{KeyCode, KeyModifiers};
use log::error;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

/// Represents the main application state.
pub struct App {
    /// Configuration manager
    config_manager: ConfigManager,
    /// Email storage
    storage: EmailStorage,
    /// Active accounts
    accounts: Vec<Arc<Mutex<Account>>>,
    /// IMAP clients for each account
    imap_clients: Vec<Arc<Mutex<ImapClient>>>,
    /// POP3 clients for each account
    pop3_clients: Vec<Arc<Mutex<Pop3Client>>>,
    /// SMTP clients for each account
    smtp_clients: Vec<Arc<Mutex<SmtpClient>>>,
    /// Currently selected account index
    selected_account: Option<usize>,
    /// Currently selected folder
    selected_folder: String,
    /// Currently selected email index
    selected_email: Option<usize>,
    /// Currently displayed emails
    emails: Vec<Email>,
    /// Currently viewed email
    viewed_email: Option<Email>,
    /// Application running state
    running: bool,
    /// Current view
    current_view: View,
    /// Status message
    status_message: Option<String>,
    /// Base directory for configuration and storage
    #[allow(dead_code)]
    base_dir: PathBuf,
    /// Email body scroll offset
    email_scroll_offset: u16,
}

/// Represents the different views in the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    /// Account list view
    Accounts,
    /// Folder list view
    Folders,
    /// Email list view
    Emails,
    /// Email detail view
    EmailDetail,
    /// Compose email view
    ComposeEmail,
    /// Settings view
    #[allow(dead_code)]
    Settings,
}

impl App {
    /// Creates a new App instance.
    ///
    /// # Returns
    /// A Result containing the App or an error
    pub fn new() -> Result<Self> {
        // Determine base directory
        let base_dir = dirs::config_dir()
            .ok_or_else(|| anyhow!("Could not determine config directory"))?
            .join("linksy");
            
        // Create directories if they don't exist
        std::fs::create_dir_all(&base_dir)?;
        std::fs::create_dir_all(base_dir.join("storage"))?;
        
        // Create configuration manager
        let config_path = base_dir.join("config.json");
        let config_manager = ConfigManager::new(config_path.to_str().unwrap())?;
        
        // Create storage
        let storage_path = base_dir.join("storage");
        let storage = EmailStorage::new(&storage_path)?;
        
        Ok(Self {
            config_manager,
            storage,
            accounts: Vec::new(),
            imap_clients: Vec::new(),
            pop3_clients: Vec::new(),
            smtp_clients: Vec::new(),
            selected_account: None,
            selected_folder: "INBOX".to_string(),
            selected_email: None,
            emails: Vec::new(),
            viewed_email: None,
            running: true,
            current_view: View::Accounts,
            status_message: None,
            base_dir,
            email_scroll_offset: 0,
        })
    }
    
    /// Runs the application.
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub async fn run(&mut self) -> Result<()> {
        // Initialize terminal
        let mut terminal = init_terminal()?;
        
        // Load accounts
        self.load_accounts().await?;
        
        // Main event loop
        while self.running {
            // Draw UI
            terminal.draw(|f| self.draw(f))?;
            
            // Handle input
            if let Some(key) = wait_for_key(Some(Duration::from_millis(100)))? {
                self.handle_input(key).await?;
            }
            
            // Check for new emails periodically
            // TODO: Implement background checking
        }
        
        // Disconnect clients
        self.disconnect_all_clients().await?;
        
        // Restore terminal
        restore_terminal(terminal)?;
        
        Ok(())
    }
    
    /// Loads accounts from configuration.
    ///
    /// # Returns
    /// A Result indicating success or failure
    async fn load_accounts(&mut self) -> Result<()> {
        // Get accounts from configuration
        let config = self.config_manager.get_config();
        
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
        
        // Select first account if available
        if !self.accounts.is_empty() {
            self.selected_account = Some(0);
            self.connect_selected_account().await?;
        }
        
        Ok(())
    }
    
    /// Connects to the selected account.
    ///
    /// # Returns
    /// A Result indicating success or failure
    async fn connect_selected_account(&mut self) -> Result<()> {
        if let Some(index) = self.selected_account {
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
                    self.set_status_message(error_msg);
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
                    self.set_status_message(error_msg);
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
                self.set_status_message(error_msg);
            }
            
            // Load emails
            self.load_emails().await?;
        }
        
        Ok(())
    }
    
    /// Disconnects all clients.
    ///
    /// # Returns
    /// A Result indicating success or failure
    async fn disconnect_all_clients(&mut self) -> Result<()> {
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
    
    /// Loads emails for the selected account and folder.
    ///
    /// # Returns
    /// A Result indicating success or failure
    async fn load_emails(&mut self) -> Result<()> {
        if let Some(index) = self.selected_account {
            let account = &self.accounts[index];
            
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
            match self.storage.get_emails(&account_id, &self.selected_folder) {
                Ok(emails) => {
                    if !emails.is_empty() {
                        self.emails = emails;
                    }
                },
                Err(e) => {
                    error!("Failed to load emails from storage: {}", e);
                }
            }
            
            // Check if account has IMAP and is connected
            if has_imap && imap_status == ConnectionStatus::Connected {
                // Fetch emails from IMAP
                let imap_client = &self.imap_clients[index];
                let client = imap_client.lock().await;
                
                match client.fetch_emails(&self.selected_folder, 50).await {
                    Ok(emails) => {
                        // Store emails in storage
                        for email in &emails {
                            if let Err(e) = self.storage.store_email(email) {
                                error!("Failed to store email: {}", e);
                            }
                        }
                        
                        self.emails = emails;
                    },
                    Err(e) => {
                        let error_msg = format!("Failed to fetch emails: {}", e);
                        drop(client); // Release the lock before modifying self
                        self.set_status_message(error_msg);
                    }
                }
            } else if has_pop3 && pop3_status == ConnectionStatus::Connected {
                // Fetch emails from POP3
                let pop3_client = &self.pop3_clients[index];
                let client = pop3_client.lock().await;
                
                match client.fetch_emails(50).await {
                    Ok(emails) => {
                        // Store emails in storage
                        for email in &emails {
                            if let Err(e) = self.storage.store_email(email) {
                                error!("Failed to store email: {}", e);
                            }
                        }
                        
                        self.emails = emails;
                    },
                    Err(e) => {
                        let error_msg = format!("Failed to fetch emails: {}", e);
                        drop(client); // Release the lock before modifying self
                        self.set_status_message(error_msg);
                    }
                }
            }
        }
        
        // Reset selected email
        self.selected_email = if self.emails.is_empty() { None } else { Some(0) };
        self.viewed_email = None;
        
        Ok(())
    }
    
    /// Handles user input.
    ///
    /// # Parameters
    /// - `key`: The key event
    ///
    /// # Returns
    /// A Result indicating success or failure
    async fn handle_input(&mut self, key: crossterm::event::KeyEvent) -> Result<()> {
        // Check for global keys
        if is_key_with_modifier(&key, KeyCode::Char('q'), KeyModifiers::CONTROL) {
            // Quit application
            self.running = false;
            return Ok(());
        }
        
        // Handle view-specific keys
        match self.current_view {
            View::Accounts => self.handle_accounts_input(key).await?,
            View::Folders => self.handle_folders_input(key).await?,
            View::Emails => self.handle_emails_input(key).await?,
            View::EmailDetail => self.handle_email_detail_input(key).await?,
            View::ComposeEmail => self.handle_compose_email_input(key).await?,
            View::Settings => self.handle_settings_input(key).await?,
        }
        
        Ok(())
    }
    
    /// Handles input in the accounts view.
    ///
    /// # Parameters
    /// - `key`: The key event
    ///
    /// # Returns
    /// A Result indicating success or failure
    async fn handle_accounts_input(&mut self, key: crossterm::event::KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Up => {
                // Move selection up
                if let Some(index) = self.selected_account {
                    if index > 0 {
                        self.selected_account = Some(index - 1);
                    }
                }
            },
            KeyCode::Down => {
                // Move selection down
                if let Some(index) = self.selected_account {
                    if index < self.accounts.len() - 1 {
                        self.selected_account = Some(index + 1);
                    }
                }
            },
            KeyCode::Enter => {
                // Select account and switch to folders view
                if self.selected_account.is_some() {
                    self.connect_selected_account().await?;
                    self.current_view = View::Folders;
                }
            },
            KeyCode::Char('a') => {
                // Add new account
                // TODO: Implement account creation UI
                self.set_status_message("Account creation not implemented yet".to_string());
            },
            KeyCode::Char('d') => {
                // Delete selected account
                // TODO: Implement account deletion
                self.set_status_message("Account deletion not implemented yet".to_string());
            },
            _ => {}
        }
        
        Ok(())
    }
    
    /// Handles input in the folders view.
    ///
    /// # Parameters
    /// - `key`: The key event
    ///
    /// # Returns
    /// A Result indicating success or failure
    async fn handle_folders_input(&mut self, key: crossterm::event::KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Up => {
                // Move selection up
                // TODO: Implement folder selection
            },
            KeyCode::Down => {
                // Move selection down
                // TODO: Implement folder selection
            },
            KeyCode::Enter => {
                // Select folder and switch to emails view
                self.load_emails().await?;
                self.current_view = View::Emails;
            },
            KeyCode::Esc => {
                // Go back to accounts view
                self.current_view = View::Accounts;
            },
            _ => {}
        }
        
        Ok(())
    }
    
    /// Handles input in the emails view.
    ///
    /// # Parameters
    /// - `key`: The key event
    ///
    /// # Returns
    /// A Result indicating success or failure
    async fn handle_emails_input(&mut self, key: crossterm::event::KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Up => {
                // Move selection up
                if let Some(index) = self.selected_email {
                    if index > 0 {
                        self.selected_email = Some(index - 1);
                    }
                }
            },
            KeyCode::Down => {
                // Move selection down
                if let Some(index) = self.selected_email {
                    if index < self.emails.len() - 1 {
                        self.selected_email = Some(index + 1);
                    }
                }
            },
            KeyCode::Enter => {
                // View selected email
                if let Some(index) = self.selected_email {
                    if index < self.emails.len() {
                        self.viewed_email = Some(self.emails[index].clone());
                        self.current_view = View::EmailDetail;
                        
                        // Mark as read if using IMAP
                        if let Some(account_index) = self.selected_account {
                            let account = &self.accounts[account_index];
                            let account_lock = account.lock().await;
                            
                            if account_lock.has_imap() && account_lock.imap_status == ConnectionStatus::Connected {
                                let imap_client = &self.imap_clients[account_index];
                                let client = imap_client.lock().await;
                                
                                let email = &self.emails[index];
                                if !email.is_read {
                                    if let Err(e) = client.mark_as_read(&self.selected_folder, &email.id).await {
                                        error!("Failed to mark email as read: {}", e);
                                    } else {
                                        // Update email in storage
                                        let mut updated_email = email.clone();
                                        updated_email.is_read = true;
                                        if let Err(e) = self.storage.update_email(&updated_email) {
                                            error!("Failed to update email in storage: {}", e);
                                        }
                                        
                                        // Update email in memory
                                        self.emails[index].is_read = true;
                                    }
                                }
                            }
                        }
                    }
                }
            },
            KeyCode::Char('c') => {
                // Compose new email
                self.current_view = View::ComposeEmail;
                // TODO: Initialize compose state
            },
            KeyCode::Char('r') => {
                // Reply to selected email
                // TODO: Implement reply
                self.set_status_message("Reply not implemented yet".to_string());
            },
            KeyCode::Char('f') => {
                // Forward selected email
                // TODO: Implement forward
                self.set_status_message("Forward not implemented yet".to_string());
            },
            KeyCode::Char('d') => {
                // Delete selected email
                // TODO: Implement delete
                self.set_status_message("Delete not implemented yet".to_string());
            },
            KeyCode::Esc => {
                // Go back to folders view
                self.current_view = View::Folders;
            },
            _ => {}
        }
        
        Ok(())
    }
    
    /// Handles input in the email detail view.
    ///
    /// # Parameters
    /// - `key`: The key event
    ///
    /// # Returns
    /// A Result indicating success or failure
    async fn handle_email_detail_input(&mut self, key: crossterm::event::KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                // Go back to emails view
                self.current_view = View::Emails;
                // Reset scroll position
                self.email_scroll_offset = 0;
            },
            KeyCode::Up => {
                // Scroll up
                if self.email_scroll_offset > 0 {
                    self.email_scroll_offset -= 1;
                }
            },
            KeyCode::Down => {
                // Scroll down
                self.email_scroll_offset += 1;
                // The render function will handle clamping to max scroll
            },
            KeyCode::PageUp => {
                // Scroll up by a page
                if self.email_scroll_offset >= 10 {
                    self.email_scroll_offset -= 10;
                } else {
                    self.email_scroll_offset = 0;
                }
            },
            KeyCode::PageDown => {
                // Scroll down by a page
                self.email_scroll_offset += 10;
                // The render function will handle clamping to max scroll
            },
            KeyCode::Home => {
                // Scroll to top
                self.email_scroll_offset = 0;
            },
            KeyCode::End => {
                // Scroll to bottom - we'll set a large value and let the render function clamp it
                self.email_scroll_offset = u16::MAX / 2;
                // The render function will handle clamping to max scroll
            },
            KeyCode::Char('r') => {
                // Reply to email
                // TODO: Implement reply
                self.set_status_message("Reply not implemented yet".to_string());
            },
            KeyCode::Char('f') => {
                // Forward email
                // TODO: Implement forward
                self.set_status_message("Forward not implemented yet".to_string());
            },
            KeyCode::Char('d') => {
                // Delete email
                // TODO: Implement delete
                self.set_status_message("Delete not implemented yet".to_string());
            },
            _ => {}
        }
        
        Ok(())
    }
    
    /// Handles input in the compose email view.
    ///
    /// # Parameters
    /// - `key`: The key event
    ///
    /// # Returns
    /// A Result indicating success or failure
    async fn handle_compose_email_input(&mut self, key: crossterm::event::KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                // Cancel compose and go back to emails view
                self.current_view = View::Emails;
            },
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Send email
                // TODO: Implement send
                self.set_status_message("Send not implemented yet".to_string());
            },
            _ => {
                // Handle text input
                // TODO: Implement text input
            }
        }
        
        Ok(())
    }
    
    /// Handles input in the settings view.
    ///
    /// # Parameters
    /// - `key`: The key event
    ///
    /// # Returns
    /// A Result indicating success or failure
    async fn handle_settings_input(&mut self, key: crossterm::event::KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                // Go back to accounts view
                self.current_view = View::Accounts;
            },
            _ => {}
        }
        
        Ok(())
    }
    
    /// Sets a status message.
    ///
    /// # Parameters
    /// - `message`: The message to display
    fn set_status_message(&mut self, message: String) {
        self.status_message = Some(message);
    }
    
    /// Draws the UI.
    ///
    /// # Parameters
    /// - `f`: The frame to draw on
    fn draw(&self, f: &mut Frame) {
        // Create layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3),  // Title
                Constraint::Min(0),     // Content
                Constraint::Length(3),  // Status
            ].as_ref())
            .split(f.size());
            
        // Draw title
        self.draw_title(f, chunks[0]);
        
        // Draw content based on current view
        match self.current_view {
            View::Accounts => self.draw_accounts(f, chunks[1]),
            View::Folders => self.draw_folders(f, chunks[1]),
            View::Emails => self.draw_emails(f, chunks[1]),
            View::EmailDetail => self.draw_email_detail(f, chunks[1]),
            View::ComposeEmail => self.draw_compose_email(f, chunks[1]),
            View::Settings => self.draw_settings(f, chunks[1]),
        }
        
        // Draw status
        self.draw_status(f, chunks[2]);
    }
    
    /// Draws the title bar.
    ///
    /// # Parameters
    /// - `f`: The frame to draw on
    /// - `area`: The area to draw in
    fn draw_title(&self, f: &mut Frame, area: Rect) {
        let title = Paragraph::new("Linksy Email Client")
            .style(Style::default().fg(Color::White))
            .block(Block::default().borders(Borders::ALL).title("Linksy"));
            
        f.render_widget(title, area);
    }
    
    /// Draws the accounts view.
    ///
    /// # Parameters
    /// - `f`: The frame to draw on
    /// - `area`: The area to draw in
    fn draw_accounts(&self, f: &mut Frame, area: Rect) {
        let accounts: Vec<ListItem> = self.accounts.iter()
            .map(|account| {
                let account = account.blocking_lock();
                ListItem::new(account.get_display_name())
            })
            .collect();
            
        let accounts_list = List::new(accounts)
            .block(Block::default().borders(Borders::ALL).title("Accounts"))
            .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .highlight_symbol("> ");
            
        let mut list_state = ListState::default();
        if let Some(i) = self.selected_account {
            list_state.select(Some(i));
        }
        
        f.render_stateful_widget(
            accounts_list,
            area,
            &mut list_state,
        );
    }
    
    /// Draws the folders view.
    ///
    /// # Parameters
    /// - `f`: The frame to draw on
    /// - `area`: The area to draw in
    fn draw_folders(&self, f: &mut Frame, area: Rect) {
        let folders = if let Some(index) = self.selected_account {
            let account = self.accounts[index].blocking_lock();
            account.folders.clone()
        } else {
            vec!["INBOX".to_string()]
        };
        
        let folder_items: Vec<ListItem> = folders.iter()
            .map(|folder| ListItem::new(folder.clone()))
            .collect();
            
        let folders_list = List::new(folder_items)
            .block(Block::default().borders(Borders::ALL).title("Folders"))
            .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .highlight_symbol("> ");
            
        // TODO: Implement folder selection state
        f.render_widget(folders_list, area);
    }
    
    /// Draws the emails view.
    ///
    /// # Parameters
    /// - `f`: The frame to draw on
    /// - `area`: The area to draw in
    fn draw_emails(&self, f: &mut Frame, area: Rect) {
        let email_items: Vec<ListItem> = self.emails.iter()
            .map(|email| {
                let style = if email.is_read {
                    Style::default()
                } else {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                };
                
                ListItem::new(email.get_summary()).style(style)
            })
            .collect();
            
        let emails_list = List::new(email_items)
            .block(Block::default().borders(Borders::ALL).title("Emails"))
            .highlight_style(Style::default().bg(Color::DarkGray))
            .highlight_symbol("> ");
            
        let mut list_state = ListState::default();
        if let Some(i) = self.selected_email {
            list_state.select(Some(i));
        }
        
        f.render_stateful_widget(
            emails_list,
            area,
            &mut list_state,
        );
    }
    
    /// Draws the email detail view.
    ///
    /// # Parameters
    /// - `f`: The frame to draw on
    /// - `area`: The area to draw in
    fn draw_email_detail(&self, f: &mut Frame, area: Rect) {
        if let Some(email) = &self.viewed_email {
            crate::ui::views::render_email_detail(f, area, email, self.email_scroll_offset);
        }
    }
    
    /// Draws the compose email view.
    ///
    /// # Parameters
    /// - `f`: The frame to draw on
    /// - `area`: The area to draw in
    fn draw_compose_email(&self, f: &mut Frame, area: Rect) {
        // TODO: Implement compose email UI
        let compose = Paragraph::new("Compose email (not implemented)")
            .block(Block::default().borders(Borders::ALL).title("Compose"));
            
        f.render_widget(compose, area);
    }
    
    /// Draws the settings view.
    ///
    /// # Parameters
    /// - `f`: The frame to draw on
    /// - `area`: The area to draw in
    fn draw_settings(&self, f: &mut Frame, area: Rect) {
        // TODO: Implement settings UI
        let settings = Paragraph::new("Settings (not implemented)")
            .block(Block::default().borders(Borders::ALL).title("Settings"));
            
        f.render_widget(settings, area);
    }
    
    /// Draws the status bar.
    ///
    /// # Parameters
    /// - `f`: The frame to draw on
    /// - `area`: The area to draw in
    fn draw_status(&self, f: &mut Frame, area: Rect) {
        let status_text = if let Some(message) = &self.status_message {
            message.clone()
        } else {
            match self.current_view {
                View::Accounts => "Accounts - [a]dd, [d]elete, [Enter] select".to_string(),
                View::Folders => "Folders - [Enter] select, [Esc] back".to_string(),
                View::Emails => "Emails - [c]ompose, [r]eply, [f]orward, [d]elete, [Enter] view, [Esc] back".to_string(),
                View::EmailDetail => "Email - [↑/↓] scroll, [PgUp/PgDn] page scroll, [Home/End] top/bottom, [r]eply, [f]orward, [d]elete, [Esc] back".to_string(),
                View::ComposeEmail => "Compose - [Ctrl+s] send, [Esc] cancel".to_string(),
                View::Settings => "Settings - [Esc] back".to_string(),
            }
        };
        
        let status = Paragraph::new(status_text)
            .style(Style::default().fg(Color::White))
            .block(Block::default().borders(Borders::ALL).title("Status"));
            
        f.render_widget(status, area);
    }
}
