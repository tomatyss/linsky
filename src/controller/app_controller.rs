//! Main application controller for the Linksy email client.

use crate::state::{AppState, AccountManager, EmailManager, View};
use crate::ui::views::account_config::AccountFormState;
use anyhow::Result;
use log::info;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Main controller for the application.
pub struct AppController {
    /// Application state
    state: Arc<Mutex<AppState>>,
    /// Account manager
    account_manager: Arc<Mutex<AccountManager>>,
    /// Email manager
    email_manager: Arc<Mutex<EmailManager>>,
}

impl AppController {
    /// Creates a new AppController instance.
    ///
    /// # Parameters
    /// - `state`: The application state
    /// - `account_manager`: The account manager
    /// - `email_manager`: The email manager
    ///
    /// # Returns
    /// A new AppController instance
    pub fn new(
        state: Arc<Mutex<AppState>>,
        account_manager: Arc<Mutex<AccountManager>>,
        email_manager: Arc<Mutex<EmailManager>>,
    ) -> Self {
        Self {
            state,
            account_manager,
            email_manager,
        }
    }
    
    /// Initializes the application.
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub async fn initialize(&self) -> Result<()> {
        // Load accounts
        self.load_accounts().await?;
        
        Ok(())
    }
    
    /// Loads accounts from configuration.
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub async fn load_accounts(&self) -> Result<()> {
        // Get configuration manager from state
        let config_manager = {
            let state = self.state.lock().await;
            state.config_manager.clone()
        };
        
        // Load accounts
        let mut account_manager = self.account_manager.lock().await;
        account_manager.load_accounts(&config_manager).await?;
        
        // Update state with accounts
        let accounts = account_manager.get_accounts().clone();
        let mut state = self.state.lock().await;
        state.accounts = accounts;
        
        // Update account summaries
        state.update_account_summaries();
        
        // Select first account if available
        if !state.accounts.is_empty() {
            state.selected_account = Some(0);
        }
        
        Ok(())
    }
    
    /// Connects to the selected account.
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub async fn connect_selected_account(&self) -> Result<()> {
        // Get selected account index
        let selected_account = {
            let state = self.state.lock().await;
            state.selected_account
        };
        
        if let Some(index) = selected_account {
            // Connect to account
            let account_manager = self.account_manager.lock().await;
            let connection_successful = account_manager.connect_account(index).await?;
            
            // If connection was successful, load emails
            if connection_successful {
                self.load_emails().await?;
            }
        }
        
        Ok(())
    }
    
    /// Loads emails for the selected account and folder.
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub async fn load_emails(&self) -> Result<()> {
        // Get selected account index and folder
        let (selected_account, folder) = {
            let state = self.state.lock().await;
            (state.selected_account, state.selected_folder.clone())
        };
        
        if let Some(index) = selected_account {
            // Get account and clients
            let account_manager = self.account_manager.lock().await;
            let account = account_manager.get_account(index).cloned();
            let imap_client = account_manager.get_imap_client(index).cloned();
            let pop3_client = account_manager.get_pop3_client(index).cloned();
            
            if let Some(account) = account {
                // Load emails
                let email_manager = self.email_manager.lock().await;
                let emails = email_manager.load_emails(
                    &account,
                    imap_client.as_ref(),
                    pop3_client.as_ref(),
                    &folder,
                    50,
                ).await?;
                
                // Update state with emails
                let mut state = self.state.lock().await;
                state.emails = emails;
                
                // Reset selected email
                state.selected_email = if state.emails.is_empty() { None } else { Some(0) };
                state.viewed_email = None;
            }
        }
        
        Ok(())
    }
    
    /// Retries failed connections for the selected account.
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub async fn retry_connections(&self) -> Result<()> {
        // Get selected account index
        let selected_account = {
            let state = self.state.lock().await;
            state.selected_account
        };
        
        if let Some(index) = selected_account {
            // Retry connections
            let account_manager = self.account_manager.lock().await;
            account_manager.retry_connections(index).await?;
            
            // Load emails
            self.load_emails().await?;
        }
        
        Ok(())
    }
    
    /// Disconnects all clients.
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub async fn disconnect_all_clients(&self) -> Result<()> {
        let account_manager = self.account_manager.lock().await;
        account_manager.disconnect_all_clients().await?;
        
        Ok(())
    }
    
    /// Marks an email as read.
    ///
    /// # Parameters
    /// - `email_index`: The index of the email to mark
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub async fn mark_email_as_read(&self, email_index: usize) -> Result<()> {
        // Get selected account index, folder, and email
        let (selected_account, folder, email) = {
            let state = self.state.lock().await;
            if email_index >= state.emails.len() {
                return Ok(());
            }
            (
                state.selected_account,
                state.selected_folder.clone(),
                state.emails[email_index].clone(),
            )
        };
        
        if let Some(index) = selected_account {
            // Get IMAP client
            let account_manager = self.account_manager.lock().await;
            let imap_client = account_manager.get_imap_client(index).cloned();
            
            if let Some(imap_client) = imap_client {
                // Mark as read
                let email_manager = self.email_manager.lock().await;
                email_manager.mark_as_read(&imap_client, &email, &folder).await?;
                
                // Update email in state
                let mut state = self.state.lock().await;
                if email_index < state.emails.len() {
                    state.emails[email_index].is_read = true;
                }
                
                // Update viewed email if it's the same
                if let Some(viewed_email) = &mut state.viewed_email {
                    if viewed_email.id == email.id {
                        viewed_email.is_read = true;
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Creates a new account form.
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub async fn create_account_form(&self) -> Result<()> {
        let mut state = self.state.lock().await;
        state.account_form_state = Some(AccountFormState::new());
        state.current_view = View::AccountConfig;
        
        Ok(())
    }
    
    /// Saves an account from the form.
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub async fn save_account_form(&self) -> Result<()> {
        // Get form state and check if it's valid
        let state = self.state.lock().await;
        let form_state_opt = state.account_form_state.clone();
        drop(state);
        
        let (form_state, is_new_account) = if let Some(mut form_state) = form_state_opt {
            if !form_state.validate() {
                return Ok(());
            }
            (form_state.clone(), form_state.is_new_account)
        } else {
            return Ok(());
        };
        
        // Finalize account
        let account_config = form_state.finalize_account();
        
        // Get configuration manager from state
        let mut config_manager = {
            let state = self.state.lock().await;
            state.config_manager.clone()
        };
        
        // Add or update account
        let mut account_manager = self.account_manager.lock().await;
        if is_new_account {
            // Add new account
            account_manager.add_account(account_config, &mut config_manager).await?;
        } else {
            // Get selected account index
            let selected_account = {
                let state = self.state.lock().await;
                state.selected_account
            };
            
            if let Some(index) = selected_account {
                // Update existing account
                account_manager.update_account(index, account_config, &mut config_manager).await?;
            }
        }
        
        // Update state with accounts
        let accounts = account_manager.get_accounts().clone();
        let mut state = self.state.lock().await;
        state.accounts = accounts;
        
        // Update account summaries
        state.update_account_summaries();
        
        // Clear form state and go back to accounts view
        state.account_form_state = None;
        state.current_view = View::Accounts;
        
        // Set status message
        state.set_status_message("Account saved successfully".to_string());
        
        Ok(())
    }
    
    /// Deletes the selected account.
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub async fn delete_selected_account(&self) -> Result<()> {
        // Get selected account index
        let selected_account = {
            let state = self.state.lock().await;
            state.selected_account
        };
        
        if let Some(index) = selected_account {
            // Get configuration manager from state
            let mut config_manager = {
                let state = self.state.lock().await;
                state.config_manager.clone()
            };
            
            // Delete account
            let mut account_manager = self.account_manager.lock().await;
            account_manager.delete_account(index, &mut config_manager).await?;
            
            // Update state with accounts
            let accounts = account_manager.get_accounts().clone();
            let mut state = self.state.lock().await;
            state.accounts = accounts;
            
            // Update account summaries
            state.update_account_summaries();
            
            // Update selected account
            if state.accounts.is_empty() {
                state.selected_account = None;
            } else if index >= state.accounts.len() {
                state.selected_account = Some(state.accounts.len() - 1);
            }
            
            // Set status message
            state.set_status_message("Account deleted successfully".to_string());
        }
        
        Ok(())
    }
    
    /// Closes the database storage.
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub async fn close_storage(&self) -> Result<()> {
        info!("Closing database storage");
        
        // Close storage in app state
        let state = self.state.lock().await;
        state.storage.close()?;
        
        // Close storage in email manager
        let email_manager = self.email_manager.lock().await;
        email_manager.close_storage()?;
        
        Ok(())
    }
    
    /// Shuts down the application.
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down Linksy email client");
        
        // Disconnect all clients
        self.disconnect_all_clients().await?;
        
        // Close database storage
        self.close_storage().await?;
        
        // Set running state to false
        let mut state = self.state.lock().await;
        state.running = false;
        
        Ok(())
    }
}
