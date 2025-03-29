//! Account configuration view for the Linksy email client.
//! 
//! This module contains the UI implementation for adding and editing email accounts.

use crate::config::{EmailAccount, ServerConfig};
use tui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Represents the state of the account configuration form.
pub struct AccountFormState {
    /// The account being configured
    pub account: EmailAccount,
    /// Index of the currently selected field
    pub selected_field: usize,
    /// Whether the selected field is being edited
    pub editing: bool,
    /// Buffer for editing text fields
    pub edit_buffer: String,
    /// Validation errors for fields
    pub validation_errors: std::collections::HashMap<String, String>,
    /// Whether this is a new account or editing an existing one
    pub is_new_account: bool,
    /// Whether IMAP settings are enabled
    pub imap_enabled: bool,
    /// Whether POP3 settings are enabled
    pub pop3_enabled: bool,
}

impl AccountFormState {
    /// Creates a new AccountFormState for a new account.
    ///
    /// # Returns
    /// A new AccountFormState instance
    pub fn new() -> Self {
        // Create default SMTP config
        let smtp_config = ServerConfig {
            host: String::new(),
            port: 587,
            username: String::new(),
            password: String::new(),
            use_ssl: true,
        };
        
        // Create default account
        let account = EmailAccount {
            id: String::new(),
            name: String::new(),
            email: String::new(),
            imap: None,
            pop3: None,
            smtp: smtp_config,
        };
        
        Self {
            account,
            selected_field: 0,
            editing: false,
            edit_buffer: String::new(),
            validation_errors: std::collections::HashMap::new(),
            is_new_account: true,
            imap_enabled: false,
            pop3_enabled: false,
        }
    }
    
    /// Creates a new AccountFormState for editing an existing account.
    ///
    /// # Parameters
    /// - `account`: The account to edit
    ///
    /// # Returns
    /// A new AccountFormState instance
    pub fn from_account(account: EmailAccount) -> Self {
        let imap_enabled = account.imap.is_some();
        let pop3_enabled = account.pop3.is_some();
        
        Self {
            account,
            selected_field: 0,
            editing: false,
            edit_buffer: String::new(),
            validation_errors: std::collections::HashMap::new(),
            is_new_account: false,
            imap_enabled,
            pop3_enabled,
        }
    }
    
    /// Gets the field name for the currently selected field.
    ///
    /// # Returns
    /// The field name as a string
    pub fn get_selected_field_name(&self) -> &'static str {
        match self.selected_field {
            0 => "account_id",
            1 => "account_name",
            2 => "email",
            3 => "imap_enabled",
            4 => "imap_host",
            5 => "imap_port",
            6 => "imap_username",
            7 => "imap_password",
            8 => "imap_ssl",
            9 => "pop3_enabled",
            10 => "pop3_host",
            11 => "pop3_port",
            12 => "pop3_username",
            13 => "pop3_password",
            14 => "pop3_ssl",
            15 => "smtp_host",
            16 => "smtp_port",
            17 => "smtp_username",
            18 => "smtp_password",
            19 => "smtp_ssl",
            20 => "save_button",
            21 => "cancel_button",
            _ => "unknown",
        }
    }
    
    /// Gets the current value of the selected field.
    ///
    /// # Returns
    /// The field value as a string
    pub fn get_selected_field_value(&self) -> String {
        match self.selected_field {
            0 => self.account.id.clone(),
            1 => self.account.name.clone(),
            2 => self.account.email.clone(),
            3 => if self.imap_enabled { "Yes" } else { "No" }.to_string(),
            4 => self.account.imap.as_ref().map_or(String::new(), |c| c.host.clone()),
            5 => self.account.imap.as_ref().map_or("993".to_string(), |c| c.port.to_string()),
            6 => self.account.imap.as_ref().map_or(String::new(), |c| c.username.clone()),
            7 => self.account.imap.as_ref().map_or(String::new(), |c| c.password.clone()),
            8 => self.account.imap.as_ref().map_or("Yes".to_string(), |c| if c.use_ssl { "Yes" } else { "No" }.to_string()),
            9 => if self.pop3_enabled { "Yes" } else { "No" }.to_string(),
            10 => self.account.pop3.as_ref().map_or(String::new(), |c| c.host.clone()),
            11 => self.account.pop3.as_ref().map_or("995".to_string(), |c| c.port.to_string()),
            12 => self.account.pop3.as_ref().map_or(String::new(), |c| c.username.clone()),
            13 => self.account.pop3.as_ref().map_or(String::new(), |c| c.password.clone()),
            14 => self.account.pop3.as_ref().map_or("Yes".to_string(), |c| if c.use_ssl { "Yes" } else { "No" }.to_string()),
            15 => self.account.smtp.host.clone(),
            16 => self.account.smtp.port.to_string(),
            17 => self.account.smtp.username.clone(),
            18 => self.account.smtp.password.clone(),
            19 => if self.account.smtp.use_ssl { "Yes" } else { "No" }.to_string(),
            20 => "Save".to_string(),
            21 => "Cancel".to_string(),
            _ => String::new(),
        }
    }
    
    /// Sets the value of the selected field.
    ///
    /// # Parameters
    /// - `value`: The new value for the field
    pub fn set_selected_field_value(&mut self, value: String) {
        match self.selected_field {
            0 => self.account.id = value,
            1 => self.account.name = value,
            2 => self.account.email = value,
            3 => self.imap_enabled = value.to_lowercase() == "yes" || value == "1" || value.to_lowercase() == "true",
            4 => {
                if self.imap_enabled {
                    let mut imap = self.account.imap.take().unwrap_or_else(|| ServerConfig {
                        host: String::new(),
                        port: 993,
                        username: self.account.email.clone(),
                        password: String::new(),
                        use_ssl: true,
                    });
                    imap.host = value;
                    self.account.imap = Some(imap);
                }
            },
            5 => {
                if self.imap_enabled {
                    if let Ok(port) = value.parse::<u16>() {
                        let mut imap = self.account.imap.take().unwrap_or_else(|| ServerConfig {
                            host: String::new(),
                            port: 993,
                            username: self.account.email.clone(),
                            password: String::new(),
                            use_ssl: true,
                        });
                        imap.port = port;
                        self.account.imap = Some(imap);
                    }
                }
            },
            6 => {
                if self.imap_enabled {
                    let mut imap = self.account.imap.take().unwrap_or_else(|| ServerConfig {
                        host: String::new(),
                        port: 993,
                        username: self.account.email.clone(),
                        password: String::new(),
                        use_ssl: true,
                    });
                    imap.username = value;
                    self.account.imap = Some(imap);
                }
            },
            7 => {
                if self.imap_enabled {
                    let mut imap = self.account.imap.take().unwrap_or_else(|| ServerConfig {
                        host: String::new(),
                        port: 993,
                        username: self.account.email.clone(),
                        password: String::new(),
                        use_ssl: true,
                    });
                    imap.password = value;
                    self.account.imap = Some(imap);
                }
            },
            8 => {
                if self.imap_enabled {
                    let use_ssl = value.to_lowercase() == "yes" || value == "1" || value.to_lowercase() == "true";
                    let mut imap = self.account.imap.take().unwrap_or_else(|| ServerConfig {
                        host: String::new(),
                        port: 993,
                        username: self.account.email.clone(),
                        password: String::new(),
                        use_ssl: true,
                    });
                    imap.use_ssl = use_ssl;
                    self.account.imap = Some(imap);
                }
            },
            9 => self.pop3_enabled = value.to_lowercase() == "yes" || value == "1" || value.to_lowercase() == "true",
            10 => {
                if self.pop3_enabled {
                    let mut pop3 = self.account.pop3.take().unwrap_or_else(|| ServerConfig {
                        host: String::new(),
                        port: 995,
                        username: self.account.email.clone(),
                        password: String::new(),
                        use_ssl: true,
                    });
                    pop3.host = value;
                    self.account.pop3 = Some(pop3);
                }
            },
            11 => {
                if self.pop3_enabled {
                    if let Ok(port) = value.parse::<u16>() {
                        let mut pop3 = self.account.pop3.take().unwrap_or_else(|| ServerConfig {
                            host: String::new(),
                            port: 995,
                            username: self.account.email.clone(),
                            password: String::new(),
                            use_ssl: true,
                        });
                        pop3.port = port;
                        self.account.pop3 = Some(pop3);
                    }
                }
            },
            12 => {
                if self.pop3_enabled {
                    let mut pop3 = self.account.pop3.take().unwrap_or_else(|| ServerConfig {
                        host: String::new(),
                        port: 995,
                        username: self.account.email.clone(),
                        password: String::new(),
                        use_ssl: true,
                    });
                    pop3.username = value;
                    self.account.pop3 = Some(pop3);
                }
            },
            13 => {
                if self.pop3_enabled {
                    let mut pop3 = self.account.pop3.take().unwrap_or_else(|| ServerConfig {
                        host: String::new(),
                        port: 995,
                        username: self.account.email.clone(),
                        password: String::new(),
                        use_ssl: true,
                    });
                    pop3.password = value;
                    self.account.pop3 = Some(pop3);
                }
            },
            14 => {
                if self.pop3_enabled {
                    let use_ssl = value.to_lowercase() == "yes" || value == "1" || value.to_lowercase() == "true";
                    let mut pop3 = self.account.pop3.take().unwrap_or_else(|| ServerConfig {
                        host: String::new(),
                        port: 995,
                        username: self.account.email.clone(),
                        password: String::new(),
                        use_ssl: true,
                    });
                    pop3.use_ssl = use_ssl;
                    self.account.pop3 = Some(pop3);
                }
            },
            15 => self.account.smtp.host = value,
            16 => {
                if let Ok(port) = value.parse::<u16>() {
                    self.account.smtp.port = port;
                }
            },
            17 => self.account.smtp.username = value,
            18 => self.account.smtp.password = value,
            19 => self.account.smtp.use_ssl = value.to_lowercase() == "yes" || value == "1" || value.to_lowercase() == "true",
            _ => {}
        }
        
        // Update IMAP and POP3 settings based on enabled flags
        if !self.imap_enabled {
            self.account.imap = None;
        } else if self.account.imap.is_none() {
            self.account.imap = Some(ServerConfig {
                host: String::new(),
                port: 993,
                username: self.account.email.clone(),
                password: String::new(),
                use_ssl: true,
            });
        }
        
        if !self.pop3_enabled {
            self.account.pop3 = None;
        } else if self.account.pop3.is_none() {
            self.account.pop3 = Some(ServerConfig {
                host: String::new(),
                port: 995,
                username: self.account.email.clone(),
                password: String::new(),
                use_ssl: true,
            });
        }
    }
    
    /// Validates the form fields.
    ///
    /// # Returns
    /// true if all fields are valid, false otherwise
    pub fn validate(&mut self) -> bool {
        self.validation_errors.clear();
        
        // Validate account ID
        if self.account.id.is_empty() {
            self.validation_errors.insert("account_id".to_string(), "Account ID is required".to_string());
        }
        
        // Validate account name
        if self.account.name.is_empty() {
            self.validation_errors.insert("account_name".to_string(), "Account name is required".to_string());
        }
        
        // Validate email
        if self.account.email.is_empty() {
            self.validation_errors.insert("email".to_string(), "Email is required".to_string());
        } else if !self.account.email.contains('@') {
            self.validation_errors.insert("email".to_string(), "Invalid email format".to_string());
        }
        
        // Validate IMAP settings if enabled
        if self.imap_enabled {
            if let Some(imap) = &self.account.imap {
                if imap.host.is_empty() {
                    self.validation_errors.insert("imap_host".to_string(), "IMAP host is required".to_string());
                }
                if imap.username.is_empty() {
                    self.validation_errors.insert("imap_username".to_string(), "IMAP username is required".to_string());
                }
                if imap.password.is_empty() {
                    self.validation_errors.insert("imap_password".to_string(), "IMAP password is required".to_string());
                }
            } else {
                self.validation_errors.insert("imap_enabled".to_string(), "IMAP settings are incomplete".to_string());
            }
        }
        
        // Validate POP3 settings if enabled
        if self.pop3_enabled {
            if let Some(pop3) = &self.account.pop3 {
                if pop3.host.is_empty() {
                    self.validation_errors.insert("pop3_host".to_string(), "POP3 host is required".to_string());
                }
                if pop3.username.is_empty() {
                    self.validation_errors.insert("pop3_username".to_string(), "POP3 username is required".to_string());
                }
                if pop3.password.is_empty() {
                    self.validation_errors.insert("pop3_password".to_string(), "POP3 password is required".to_string());
                }
            } else {
                self.validation_errors.insert("pop3_enabled".to_string(), "POP3 settings are incomplete".to_string());
            }
        }
        
        // Validate SMTP settings (required)
        if self.account.smtp.host.is_empty() {
            self.validation_errors.insert("smtp_host".to_string(), "SMTP host is required".to_string());
        }
        if self.account.smtp.username.is_empty() {
            self.validation_errors.insert("smtp_username".to_string(), "SMTP username is required".to_string());
        }
        if self.account.smtp.password.is_empty() {
            self.validation_errors.insert("smtp_password".to_string(), "SMTP password is required".to_string());
        }
        
        self.validation_errors.is_empty()
    }
    
    /// Gets the validation error for a field.
    ///
    /// # Parameters
    /// - `field_name`: The name of the field
    ///
    /// # Returns
    /// An Option containing the validation error message
    pub fn get_validation_error(&self, field_name: &str) -> Option<&String> {
        self.validation_errors.get(field_name)
    }
    
    /// Checks if a field has a validation error.
    ///
    /// # Parameters
    /// - `field_name`: The name of the field
    ///
    /// # Returns
    /// true if the field has a validation error, false otherwise
    pub fn has_validation_error(&self, field_name: &str) -> bool {
        self.validation_errors.contains_key(field_name)
    }
    
    /// Starts editing the selected field.
    pub fn start_editing(&mut self) {
        self.editing = true;
        self.edit_buffer = self.get_selected_field_value();
    }
    
    /// Stops editing the selected field and applies the changes.
    pub fn stop_editing(&mut self) {
        self.editing = false;
        self.set_selected_field_value(self.edit_buffer.clone());
        self.edit_buffer.clear();
    }
    
    /// Cancels editing the selected field without applying changes.
    pub fn cancel_editing(&mut self) {
        self.editing = false;
        self.edit_buffer.clear();
    }
    
    /// Moves the selection to the previous field.
    pub fn select_previous_field(&mut self) {
        if self.selected_field > 0 {
            self.selected_field -= 1;
        } else {
            self.selected_field = 21; // Wrap to the last field
        }
        
        // Skip IMAP fields if IMAP is disabled
        if !self.imap_enabled && self.selected_field >= 4 && self.selected_field <= 8 {
            self.selected_field = 3;
        }
        
        // Skip POP3 fields if POP3 is disabled
        if !self.pop3_enabled && self.selected_field >= 10 && self.selected_field <= 14 {
            self.selected_field = 9;
        }
    }
    
    /// Moves the selection to the next field.
    pub fn select_next_field(&mut self) {
        if self.selected_field < 21 {
            self.selected_field += 1;
        } else {
            self.selected_field = 0; // Wrap to the first field
        }
        
        // Skip IMAP fields if IMAP is disabled
        if !self.imap_enabled && self.selected_field >= 4 && self.selected_field <= 8 {
            self.selected_field = 9;
        }
        
        // Skip POP3 fields if POP3 is disabled
        if !self.pop3_enabled && self.selected_field >= 10 && self.selected_field <= 14 {
            self.selected_field = 15;
        }
    }
    
    /// Toggles a boolean field.
    pub fn toggle_boolean_field(&mut self) {
        match self.selected_field {
            3 => { // IMAP enabled
                self.imap_enabled = !self.imap_enabled;
                if self.imap_enabled && self.account.imap.is_none() {
                    self.account.imap = Some(ServerConfig {
                        host: String::new(),
                        port: 993,
                        username: self.account.email.clone(),
                        password: String::new(),
                        use_ssl: true,
                    });
                }
            },
            8 => { // IMAP SSL
                if let Some(imap) = &mut self.account.imap {
                    imap.use_ssl = !imap.use_ssl;
                }
            },
            9 => { // POP3 enabled
                self.pop3_enabled = !self.pop3_enabled;
                if self.pop3_enabled && self.account.pop3.is_none() {
                    self.account.pop3 = Some(ServerConfig {
                        host: String::new(),
                        port: 995,
                        username: self.account.email.clone(),
                        password: String::new(),
                        use_ssl: true,
                    });
                }
            },
            14 => { // POP3 SSL
                if let Some(pop3) = &mut self.account.pop3 {
                    pop3.use_ssl = !pop3.use_ssl;
                }
            },
            19 => { // SMTP SSL
                self.account.smtp.use_ssl = !self.account.smtp.use_ssl;
            },
            _ => {}
        }
    }
    
    /// Finalizes the account configuration.
    ///
    /// # Returns
    /// The configured EmailAccount
    pub fn finalize_account(&self) -> EmailAccount {
        let mut account = self.account.clone();
        
        // Set IMAP and POP3 based on enabled flags
        if !self.imap_enabled {
            account.imap = None;
        }
        
        if !self.pop3_enabled {
            account.pop3 = None;
        }
        
        account
    }
}

/// Renders the account configuration view.
///
/// # Parameters
/// - `f`: The frame to render on
/// - `area`: The area to render in
/// - `form_state`: The form state
pub fn render_account_config(
    f: &mut Frame,
    area: Rect,
    form_state: &AccountFormState,
) {
    // Create layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(7),   // Basic account info
            Constraint::Length(8),   // IMAP settings
            Constraint::Length(8),   // POP3 settings
            Constraint::Length(8),   // SMTP settings
            Constraint::Length(3),   // Buttons
        ].as_ref())
        .split(area);
        
    // Render basic account info
    render_basic_info(f, chunks[0], form_state);
    
    // Render IMAP settings
    render_imap_settings(f, chunks[1], form_state);
    
    // Render POP3 settings
    render_pop3_settings(f, chunks[2], form_state);
    
    // Render SMTP settings
    render_smtp_settings(f, chunks[3], form_state);
    
    // Render buttons
    render_buttons(f, chunks[4], form_state);
}

/// Renders the basic account information section.
///
/// # Parameters
/// - `f`: The frame to render on
/// - `area`: The area to render in
/// - `form_state`: The form state
fn render_basic_info(
    f: &mut Frame,
    area: Rect,
    form_state: &AccountFormState,
) {
    let title = if form_state.is_new_account {
        "Add New Account"
    } else {
        "Edit Account"
    };
    
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title);
        
    f.render_widget(block, area);
    
    // Create layout for fields
    let inner_area = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(1),   // Account ID
            Constraint::Length(1),   // Account Name
            Constraint::Length(1),   // Email
        ].as_ref())
        .split(area);
        
    // Render fields
    render_field(f, inner_area[0], "Account ID:", &form_state.account.id, 
        form_state.selected_field == 0, 
        form_state.editing && form_state.selected_field == 0,
        &form_state.edit_buffer,
        form_state.get_validation_error("account_id"));
        
    render_field(f, inner_area[1], "Account Name:", &form_state.account.name, 
        form_state.selected_field == 1, 
        form_state.editing && form_state.selected_field == 1,
        &form_state.edit_buffer,
        form_state.get_validation_error("account_name"));
        
    render_field(f, inner_area[2], "Email Address:", &form_state.account.email, 
        form_state.selected_field == 2, 
        form_state.editing && form_state.selected_field == 2,
        &form_state.edit_buffer,
        form_state.get_validation_error("email"));
}

/// Renders the IMAP settings section.
///
/// # Parameters
/// - `f`: The frame to render on
/// - `area`: The area to render in
/// - `form_state`: The form state
fn render_imap_settings(
    f: &mut Frame,
    area: Rect,
    form_state: &AccountFormState,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("IMAP Settings (Optional)");
        
    f.render_widget(block, area);
    
    // Create layout for fields
    let inner_area = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(1),   // Enabled
            Constraint::Length(1),   // Host
            Constraint::Length(1),   // Port
            Constraint::Length(1),   // Username
            Constraint::Length(1),   // Password
            Constraint::Length(1),   // SSL
        ].as_ref())
        .split(area);
        
    // Render fields
    render_field(f, inner_area[0], "Enabled:", 
        if form_state.imap_enabled { "Yes" } else { "No" }, 
        form_state.selected_field == 3, 
        false,
        &form_state.edit_buffer,
        form_state.get_validation_error("imap_enabled"));
        
    // Only render IMAP fields if enabled
    if form_state.imap_enabled {
        // Create a longer-lived ServerConfig for the default values
        let default_imap = ServerConfig {
            host: String::new(),
            port: 993,
            username: String::new(),
            password: String::new(),
            use_ssl: true,
        };
        
        // Use the reference to the longer-lived value
        let imap = form_state.account.imap.as_ref().unwrap_or(&default_imap);
        
        render_field(f, inner_area[1], "Host:", &imap.host, 
            form_state.selected_field == 4, 
            form_state.editing && form_state.selected_field == 4,
            &form_state.edit_buffer,
            form_state.get_validation_error("imap_host"));
            
        render_field(f, inner_area[2], "Port:", &imap.port.to_string(), 
            form_state.selected_field == 5, 
            form_state.editing && form_state.selected_field == 5,
            &form_state.edit_buffer,
            form_state.get_validation_error("imap_port"));
            
        render_field(f, inner_area[3], "Username:", &imap.username, 
            form_state.selected_field == 6, 
            form_state.editing && form_state.selected_field == 6,
            &form_state.edit_buffer,
            form_state.get_validation_error("imap_username"));
            
        render_field(f, inner_area[4], "Password:", 
            &"*".repeat(imap.password.len().max(1)), 
            form_state.selected_field == 7, 
            form_state.editing && form_state.selected_field == 7,
            &form_state.edit_buffer,
            form_state.get_validation_error("imap_password"));
            
        render_field(f, inner_area[5], "Use SSL/TLS:", 
            if imap.use_ssl { "Yes" } else { "No" }, 
            form_state.selected_field == 8, 
            false,
            &form_state.edit_buffer,
            form_state.get_validation_error("imap_ssl"));
    }
}

/// Renders the POP3 settings section.
///
/// # Parameters
/// - `f`: The frame to render on
/// - `area`: The area to render in
/// - `form_state`: The form state
fn render_pop3_settings(
    f: &mut Frame,
    area: Rect,
    form_state: &AccountFormState,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("POP3 Settings (Optional)");
        
    f.render_widget(block, area);
    
    // Create layout for fields
    let inner_area = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(1),   // Enabled
            Constraint::Length(1),   // Host
            Constraint::Length(1),   // Port
            Constraint::Length(1),   // Username
            Constraint::Length(1),   // Password
            Constraint::Length(1),   // SSL
        ].as_ref())
        .split(area);
        
    // Render fields
    render_field(f, inner_area[0], "Enabled:", 
        if form_state.pop3_enabled { "Yes" } else { "No" }, 
        form_state.selected_field == 9, 
        false,
        &form_state.edit_buffer,
        form_state.get_validation_error("pop3_enabled"));
        
    // Only render POP3 fields if enabled
    if form_state.pop3_enabled {
        // Create a longer-lived ServerConfig for the default values
        let default_pop3 = ServerConfig {
            host: String::new(),
            port: 995,
            username: String::new(),
            password: String::new(),
            use_ssl: true,
        };
        
        // Use the reference to the longer-lived value
        let pop3 = form_state.account.pop3.as_ref().unwrap_or(&default_pop3);
        
        render_field(f, inner_area[1], "Host:", &pop3.host, 
            form_state.selected_field == 10, 
            form_state.editing && form_state.selected_field == 10,
            &form_state.edit_buffer,
            form_state.get_validation_error("pop3_host"));
            
        render_field(f, inner_area[2], "Port:", &pop3.port.to_string(), 
            form_state.selected_field == 11, 
            form_state.editing && form_state.selected_field == 11,
            &form_state.edit_buffer,
            form_state.get_validation_error("pop3_port"));
            
        render_field(f, inner_area[3], "Username:", &pop3.username, 
            form_state.selected_field == 12, 
            form_state.editing && form_state.selected_field == 12,
            &form_state.edit_buffer,
            form_state.get_validation_error("pop3_username"));
            
        render_field(f, inner_area[4], "Password:", 
            &"*".repeat(pop3.password.len().max(1)), 
            form_state.selected_field == 13, 
            form_state.editing && form_state.selected_field == 13,
            &form_state.edit_buffer,
            form_state.get_validation_error("pop3_password"));
            
        render_field(f, inner_area[5], "Use SSL/TLS:", 
            if pop3.use_ssl { "Yes" } else { "No" }, 
            form_state.selected_field == 14, 
            false,
            &form_state.edit_buffer,
            form_state.get_validation_error("pop3_ssl"));
    }
}

/// Renders the SMTP settings section.
///
/// # Parameters
/// - `f`: The frame to render on
/// - `area`: The area to render in
/// - `form_state`: The form state
fn render_smtp_settings(
    f: &mut Frame,
    area: Rect,
    form_state: &AccountFormState,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("SMTP Settings (Required)");
        
    f.render_widget(block, area);
    
    // Create layout for fields
    let inner_area = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(1),   // Host
            Constraint::Length(1),   // Port
            Constraint::Length(1),   // Username
            Constraint::Length(1),   // Password
            Constraint::Length(1),   // SSL
        ].as_ref())
        .split(area);
        
    // Render fields
    render_field(f, inner_area[0], "Host:", &form_state.account.smtp.host, 
        form_state.selected_field == 15, 
        form_state.editing && form_state.selected_field == 15,
        &form_state.edit_buffer,
        form_state.get_validation_error("smtp_host"));
        
    render_field(f, inner_area[1], "Port:", &form_state.account.smtp.port.to_string(), 
        form_state.selected_field == 16, 
        form_state.editing && form_state.selected_field == 16,
        &form_state.edit_buffer,
        form_state.get_validation_error("smtp_port"));
        
    render_field(f, inner_area[2], "Username:", &form_state.account.smtp.username, 
        form_state.selected_field == 17, 
        form_state.editing && form_state.selected_field == 17,
        &form_state.edit_buffer,
        form_state.get_validation_error("smtp_username"));
        
    render_field(f, inner_area[3], "Password:", 
        &"*".repeat(form_state.account.smtp.password.len().max(1)), 
        form_state.selected_field == 18, 
        form_state.editing && form_state.selected_field == 18,
        &form_state.edit_buffer,
        form_state.get_validation_error("smtp_password"));
        
    render_field(f, inner_area[4], "Use SSL/TLS:", 
        if form_state.account.smtp.use_ssl { "Yes" } else { "No" }, 
        form_state.selected_field == 19, 
        false,
        &form_state.edit_buffer,
        form_state.get_validation_error("smtp_ssl"));
}

/// Renders the buttons section.
///
/// # Parameters
/// - `f`: The frame to render on
/// - `area`: The area to render in
/// - `form_state`: The form state
fn render_buttons(
    f: &mut Frame,
    area: Rect,
    form_state: &AccountFormState,
) {
    let block = Block::default()
        .borders(Borders::ALL);
        
    f.render_widget(block, area);
    
    // Create layout for buttons
    let inner_area = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints([
            Constraint::Percentage(50),   // Save
            Constraint::Percentage(50),   // Cancel
        ].as_ref())
        .split(area);
        
    // Render buttons
    let save_style = if form_state.selected_field == 20 {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    
    let cancel_style = if form_state.selected_field == 21 {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    
    let save_text = Paragraph::new("Save")
        .style(save_style)
        .alignment(tui::layout::Alignment::Center);
        
    let cancel_text = Paragraph::new("Cancel")
        .style(cancel_style)
        .alignment(tui::layout::Alignment::Center);
        
    f.render_widget(save_text, inner_area[0]);
    f.render_widget(cancel_text, inner_area[1]);
}

/// Renders a form field.
///
/// # Parameters
/// - `f`: The frame to render on
/// - `area`: The area to render in
/// - `label`: The field label
/// - `value`: The field value
/// - `selected`: Whether the field is selected
/// - `editing`: Whether the field is being edited
/// - `edit_buffer`: The edit buffer for the field
/// - `error`: The validation error for the field
fn render_field(
    f: &mut Frame,
    area: Rect,
    label: &str,
    value: &str,
    selected: bool,
    editing: bool,
    edit_buffer: &str,
    error: Option<&String>,
) {
    // Calculate label width
    let label_width = 15;
    
    // Create spans for the field
    let mut spans = Vec::new();
    
    // Determine the base style for the entire line
    let base_style = if selected {
        if editing {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        }
    } else {
        Style::default()
    };
    
    // Add label with the appropriate style
    spans.push(Span::styled(
        format!("{:<width$}", label, width = label_width),
        base_style
    ));
    
    // Add value with additional underline if editing
    let display_value = if editing { edit_buffer } else { value };
    let value_style = if selected && editing {
        base_style.add_modifier(Modifier::UNDERLINED)
    } else {
        base_style
    };
    
    spans.push(Span::styled(display_value, value_style));
    
    // Add error if present (always in red regardless of selection)
    if let Some(error_msg) = error {
        spans.push(Span::styled(" ", base_style));
        spans.push(Span::styled(format!("({})", error_msg), Style::default().fg(Color::Red)));
    }
    
    // Create paragraph with a Line from the vector of spans
    let line = tui::text::Line::from(spans);
    let paragraph = Paragraph::new(line);
    
    // Render paragraph
    f.render_widget(paragraph, area);
}
