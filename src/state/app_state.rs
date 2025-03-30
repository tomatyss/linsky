//! Application state management for the Linksy email client.

use crate::config::ConfigManager;
use crate::models::{Account, Email};
use crate::storage::EmailStorage;
use crate::ui::views::account_config::AccountFormState;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

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
    /// Account configuration view
    AccountConfig,
}

/// Represents the main application state.
pub struct AppState {
    /// Configuration manager
    pub config_manager: ConfigManager,
    /// Email storage
    pub storage: EmailStorage,
    /// Active accounts
    pub accounts: Vec<Arc<Mutex<Account>>>,
    /// Currently selected account index
    pub selected_account: Option<usize>,
    /// Currently selected folder
    pub selected_folder: String,
    /// Currently selected email index
    pub selected_email: Option<usize>,
    /// Currently displayed emails
    pub emails: Vec<Email>,
    /// Currently viewed email
    pub viewed_email: Option<Email>,
    /// Application running state
    pub running: bool,
    /// Current view
    pub current_view: View,
    /// Status message
    pub status_message: Option<String>,
    /// Base directory for configuration and storage
    pub base_dir: PathBuf,
    /// Email body scroll offset
    pub email_scroll_offset: u16,
    /// Account configuration form state
    pub account_form_state: Option<AccountFormState>,
}

impl AppState {
    /// Creates a new AppState instance.
    ///
    /// # Parameters
    /// - `config_manager`: The configuration manager
    /// - `storage`: The email storage
    /// - `base_dir`: The base directory for configuration and storage
    ///
    /// # Returns
    /// A new AppState instance
    pub fn new(config_manager: ConfigManager, storage: EmailStorage, base_dir: PathBuf) -> Self {
        Self {
            config_manager,
            storage,
            accounts: Vec::new(),
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
            account_form_state: None,
        }
    }
    
    /// Sets a status message.
    ///
    /// # Parameters
    /// - `message`: The message to set
    pub fn set_status_message(&mut self, message: String) {
        self.status_message = Some(message);
    }
    
    /// Clears the status message.
    pub fn clear_status_message(&mut self) {
        self.status_message = None;
    }
    
    /// Gets the status message.
    ///
    /// # Returns
    /// An Option containing the status message
    pub fn get_status_message(&self) -> Option<&String> {
        self.status_message.as_ref()
    }
    
    /// Gets the current view.
    ///
    /// # Returns
    /// The current view
    pub fn get_current_view(&self) -> View {
        self.current_view
    }
    
    /// Sets the current view.
    ///
    /// # Parameters
    /// - `view`: The view to set
    pub fn set_current_view(&mut self, view: View) {
        self.current_view = view;
    }
    
    /// Checks if the application is running.
    ///
    /// # Returns
    /// true if the application is running, false otherwise
    pub fn is_running(&self) -> bool {
        self.running
    }
    
    /// Sets the application running state.
    ///
    /// # Parameters
    /// - `running`: The running state to set
    pub fn set_running(&mut self, running: bool) {
        self.running = running;
    }
    
    /// Gets the selected account index.
    ///
    /// # Returns
    /// An Option containing the selected account index
    pub fn get_selected_account(&self) -> Option<usize> {
        self.selected_account
    }
    
    /// Sets the selected account index.
    ///
    /// # Parameters
    /// - `index`: The index to set
    pub fn set_selected_account(&mut self, index: Option<usize>) {
        self.selected_account = index;
    }
    
    /// Gets the selected folder.
    ///
    /// # Returns
    /// A reference to the selected folder
    pub fn get_selected_folder(&self) -> &str {
        &self.selected_folder
    }
    
    /// Sets the selected folder.
    ///
    /// # Parameters
    /// - `folder`: The folder to set
    pub fn set_selected_folder(&mut self, folder: String) {
        self.selected_folder = folder;
    }
    
    /// Gets the selected email index.
    ///
    /// # Returns
    /// An Option containing the selected email index
    pub fn get_selected_email(&self) -> Option<usize> {
        self.selected_email
    }
    
    /// Sets the selected email index.
    ///
    /// # Parameters
    /// - `index`: The index to set
    pub fn set_selected_email(&mut self, index: Option<usize>) {
        self.selected_email = index;
    }
    
    /// Gets the viewed email.
    ///
    /// # Returns
    /// An Option containing a reference to the viewed email
    pub fn get_viewed_email(&self) -> Option<&Email> {
        self.viewed_email.as_ref()
    }
    
    /// Sets the viewed email.
    ///
    /// # Parameters
    /// - `email`: The email to set
    pub fn set_viewed_email(&mut self, email: Option<Email>) {
        self.viewed_email = email;
    }
    
    /// Gets the email body scroll offset.
    ///
    /// # Returns
    /// The email body scroll offset
    pub fn get_email_scroll_offset(&self) -> u16 {
        self.email_scroll_offset
    }
    
    /// Sets the email body scroll offset.
    ///
    /// # Parameters
    /// - `offset`: The offset to set
    pub fn set_email_scroll_offset(&mut self, offset: u16) {
        self.email_scroll_offset = offset;
    }
    
    /// Gets the account configuration form state.
    ///
    /// # Returns
    /// An Option containing a reference to the account configuration form state
    pub fn get_account_form_state(&self) -> Option<&AccountFormState> {
        self.account_form_state.as_ref()
    }
    
    /// Gets a mutable reference to the account configuration form state.
    ///
    /// # Returns
    /// An Option containing a mutable reference to the account configuration form state
    pub fn get_account_form_state_mut(&mut self) -> Option<&mut AccountFormState> {
        self.account_form_state.as_mut()
    }
    
    /// Sets the account configuration form state.
    ///
    /// # Parameters
    /// - `state`: The state to set
    pub fn set_account_form_state(&mut self, state: Option<AccountFormState>) {
        self.account_form_state = state;
    }
}
