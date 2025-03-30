//! Input handling for the Linksy email client.

use crate::controller::AppController;
use crate::state::View;
use crate::ui::{is_key_with_modifier};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::sync::Arc;

/// Handles user input.
pub struct InputHandler {
    /// Application controller
    controller: Arc<AppController>,
}

impl InputHandler {
    /// Creates a new InputHandler instance.
    ///
    /// # Parameters
    /// - `controller`: The application controller
    ///
    /// # Returns
    /// A new InputHandler instance
    pub fn new(controller: Arc<AppController>) -> Self {
        Self {
            controller,
        }
    }
    
    /// Handles a key event.
    ///
    /// # Parameters
    /// - `key`: The key event
    /// - `state`: The application state
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub async fn handle_key(&self, key: KeyEvent, state: &mut crate::state::AppState) -> Result<()> {
        // Check for global keys
        if is_key_with_modifier(&key, KeyCode::Char('q'), KeyModifiers::CONTROL) {
            // Quit application
            state.set_running(false);
            return Ok(());
        }
        
        // Handle view-specific keys
        match state.get_current_view() {
            View::Accounts => self.handle_accounts_input(key, state).await?,
            View::Folders => self.handle_folders_input(key, state).await?,
            View::Emails => self.handle_emails_input(key, state).await?,
            View::EmailDetail => self.handle_email_detail_input(key, state).await?,
            View::ComposeEmail => self.handle_compose_email_input(key, state).await?,
            View::Settings => self.handle_settings_input(key, state).await?,
            View::AccountConfig => self.handle_account_config_input(key, state).await?,
        }
        
        Ok(())
    }
    
    /// Handles input in the accounts view.
    ///
    /// # Parameters
    /// - `key`: The key event
    /// - `state`: The application state
    ///
    /// # Returns
    /// A Result indicating success or failure
    async fn handle_accounts_input(&self, key: KeyEvent, state: &mut crate::state::AppState) -> Result<()> {
        match key.code {
            KeyCode::Up => {
                // Move selection up
                if let Some(index) = state.get_selected_account() {
                    if index > 0 {
                        state.set_selected_account(Some(index - 1));
                    }
                }
            },
            KeyCode::Down => {
                // Move selection down
                if let Some(index) = state.get_selected_account() {
                    if index < state.accounts.len() - 1 {
                        state.set_selected_account(Some(index + 1));
                    }
                }
            },
            KeyCode::Enter => {
                // Select account and switch to folders view
                if state.get_selected_account().is_some() {
                    self.controller.connect_selected_account().await?;
                    state.set_current_view(View::Folders);
                }
            },
            KeyCode::Char('a') => {
                // Add new account
                self.controller.create_account_form().await?;
            },
            KeyCode::Char('e') => {
                // Edit selected account
                if state.get_selected_account().is_some() {
                    // Create a new empty account form state instead of trying to load the existing account
                    // This avoids blocking operations in the UI thread
                    self.controller.create_account_form().await?;
                    
                    // Set a status message to inform the user about the workaround
                    state.set_status_message("Creating new account instead of editing (blocking issue workaround)".to_string());
                }
            },
            KeyCode::Char('d') => {
                // Delete selected account
                if state.get_selected_account().is_some() {
                    self.controller.delete_selected_account().await?;
                }
            },
            _ => {}
        }
        
        Ok(())
    }
    
    /// Handles input in the account configuration view.
    ///
    /// # Parameters
    /// - `key`: The key event
    /// - `state`: The application state
    ///
    /// # Returns
    /// A Result indicating success or failure
    async fn handle_account_config_input(&self, key: KeyEvent, state: &mut crate::state::AppState) -> Result<()> {
        if let Some(form_state) = state.get_account_form_state_mut() {
            match key.code {
                KeyCode::Esc => {
                    if form_state.editing {
                        // Cancel editing the current field
                        form_state.cancel_editing();
                    } else {
                        // Go back to accounts view
                        state.set_current_view(View::Accounts);
                        state.set_account_form_state(None);
                    }
                },
                KeyCode::Up => {
                    if !form_state.editing {
                        // Move selection up
                        form_state.select_previous_field();
                    }
                },
                KeyCode::Down => {
                    if !form_state.editing {
                        // Move selection down
                        form_state.select_next_field();
                    }
                },
                KeyCode::Enter => {
                    let field_name = form_state.get_selected_field_name();
                    
                    if field_name == "save_button" {
                        // Validate and save account
                        if form_state.validate() {
                            self.controller.save_account_form().await?;
                        } else {
                            state.set_status_message("Please fix validation errors".to_string());
                        }
                    } else if field_name == "cancel_button" {
                        // Cancel and go back to accounts view
                        state.set_current_view(View::Accounts);
                        state.set_account_form_state(None);
                    } else if field_name.ends_with("_enabled") || field_name.ends_with("_ssl") {
                        // Toggle boolean fields
                        form_state.toggle_boolean_field();
                    } else {
                        // Start/stop editing the current field
                        if form_state.editing {
                            form_state.stop_editing();
                        } else {
                            form_state.start_editing();
                        }
                    }
                },
                KeyCode::Char(c) => {
                    if form_state.editing {
                        // Add character to edit buffer
                        form_state.edit_buffer.push(c);
                    }
                },
                KeyCode::Backspace => {
                    if form_state.editing {
                        // Remove character from edit buffer
                        form_state.edit_buffer.pop();
                    }
                },
                KeyCode::Tab => {
                    if !form_state.editing {
                        // Move to next field
                        form_state.select_next_field();
                    }
                },
                KeyCode::BackTab => {
                    if !form_state.editing {
                        // Move to previous field
                        form_state.select_previous_field();
                    }
                },
                _ => {}
            }
        }
        
        Ok(())
    }
    
    /// Handles input in the folders view.
    ///
    /// # Parameters
    /// - `key`: The key event
    /// - `state`: The application state
    ///
    /// # Returns
    /// A Result indicating success or failure
    async fn handle_folders_input(&self, key: KeyEvent, state: &mut crate::state::AppState) -> Result<()> {
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
                self.controller.load_emails().await?;
                state.set_current_view(View::Emails);
            },
            KeyCode::Char('r') => {
                // Retry failed connections
                state.set_status_message("Retrying failed connections...".to_string());
                self.controller.retry_connections().await?;
            },
            KeyCode::Esc => {
                // Go back to accounts view
                state.set_current_view(View::Accounts);
            },
            _ => {}
        }
        
        Ok(())
    }
    
    /// Handles input in the emails view.
    ///
    /// # Parameters
    /// - `key`: The key event
    /// - `state`: The application state
    ///
    /// # Returns
    /// A Result indicating success or failure
    async fn handle_emails_input(&self, key: KeyEvent, state: &mut crate::state::AppState) -> Result<()> {
        match key.code {
            KeyCode::Up => {
                // Move selection up
                if let Some(index) = state.get_selected_email() {
                    if index > 0 {
                        state.set_selected_email(Some(index - 1));
                    }
                }
            },
            KeyCode::Down => {
                // Move selection down
                if let Some(index) = state.get_selected_email() {
                    if index < state.emails.len() - 1 {
                        state.set_selected_email(Some(index + 1));
                    }
                }
            },
            KeyCode::Enter => {
                // View selected email
                if let Some(index) = state.get_selected_email() {
                    if index < state.emails.len() {
                        state.set_viewed_email(Some(state.emails[index].clone()));
                        state.set_current_view(View::EmailDetail);
                        state.set_email_scroll_offset(0);
                        
                        // Mark as read
                        if !state.emails[index].is_read {
                            self.controller.mark_email_as_read(index).await?;
                        }
                    }
                }
            },
            KeyCode::Char('c') => {
                // Compose new email
                state.set_current_view(View::ComposeEmail);
                // TODO: Initialize compose state
            },
            KeyCode::Char('r') => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    // Retry failed connections
                    state.set_status_message("Retrying failed connections...".to_string());
                    self.controller.retry_connections().await?;
                } else {
                    // Reply to selected email
                    // TODO: Implement reply
                    state.set_status_message("Reply not implemented yet".to_string());
                }
            },
            KeyCode::Char('f') => {
                // Forward selected email
                // TODO: Implement forward
                state.set_status_message("Forward not implemented yet".to_string());
            },
            KeyCode::Char('d') => {
                // Delete selected email
                // TODO: Implement delete
                state.set_status_message("Delete not implemented yet".to_string());
            },
            KeyCode::Esc => {
                // Go back to folders view
                state.set_current_view(View::Folders);
            },
            _ => {}
        }
        
        Ok(())
    }
    
    /// Handles input in the email detail view.
    ///
    /// # Parameters
    /// - `key`: The key event
    /// - `state`: The application state
    ///
    /// # Returns
    /// A Result indicating success or failure
    async fn handle_email_detail_input(&self, key: KeyEvent, state: &mut crate::state::AppState) -> Result<()> {
        match key.code {
            KeyCode::Up => {
                // Scroll up
                if state.get_email_scroll_offset() > 0 {
                    state.set_email_scroll_offset(state.get_email_scroll_offset() - 1);
                }
            },
            KeyCode::Down => {
                // Scroll down
                state.set_email_scroll_offset(state.get_email_scroll_offset() + 1);
            },
            KeyCode::PageUp => {
                // Scroll up by page
                if state.get_email_scroll_offset() > 10 {
                    state.set_email_scroll_offset(state.get_email_scroll_offset() - 10);
                } else {
                    state.set_email_scroll_offset(0);
                }
            },
            KeyCode::PageDown => {
                // Scroll down by page
                state.set_email_scroll_offset(state.get_email_scroll_offset() + 10);
            },
            KeyCode::Home => {
                // Scroll to top
                state.set_email_scroll_offset(0);
            },
            KeyCode::End => {
                // Scroll to bottom
                // TODO: Calculate maximum scroll offset
                state.set_email_scroll_offset(1000);
            },
            KeyCode::Char('r') => {
                // Reply to email
                // TODO: Implement reply
                state.set_status_message("Reply not implemented yet".to_string());
            },
            KeyCode::Char('f') => {
                // Forward email
                // TODO: Implement forward
                state.set_status_message("Forward not implemented yet".to_string());
            },
            KeyCode::Char('d') => {
                // Delete email
                // TODO: Implement delete
                state.set_status_message("Delete not implemented yet".to_string());
            },
            KeyCode::Esc => {
                // Go back to emails view
                state.set_current_view(View::Emails);
            },
            _ => {}
        }
        
        Ok(())
    }
    
    /// Handles input in the compose email view.
    ///
    /// # Parameters
    /// - `key`: The key event
    /// - `state`: The application state
    ///
    /// # Returns
    /// A Result indicating success or failure
    async fn handle_compose_email_input(&self, key: KeyEvent, state: &mut crate::state::AppState) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                // Cancel and go back to emails view
                state.set_current_view(View::Emails);
            },
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Send email
                // TODO: Implement send
                state.set_status_message("Send not implemented yet".to_string());
            },
            _ => {
                // TODO: Handle compose email input
            }
        }
        
        Ok(())
    }
    
    /// Handles input in the settings view.
    ///
    /// # Parameters
    /// - `key`: The key event
    /// - `state`: The application state
    ///
    /// # Returns
    /// A Result indicating success or failure
    async fn handle_settings_input(&self, key: KeyEvent, state: &mut crate::state::AppState) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                // Go back to accounts view
                state.set_current_view(View::Accounts);
            },
            _ => {}
        }
        
        Ok(())
    }
}
