//! Main application renderer for the Linksy email client.

use crate::state::{AppState, View};
use crate::ui::views;
use anyhow::Result;
use tui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Renders the application UI.
pub struct AppRenderer;

impl AppRenderer {
    /// Creates a new AppRenderer instance.
    ///
    /// # Returns
    /// A new AppRenderer instance
    pub fn new() -> Self {
        Self
    }
    
    /// Renders the application UI.
    ///
    /// # Parameters
    /// - `f`: The frame to render on
    /// - `state`: The application state
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub fn render(&self, f: &mut Frame, state: &AppState) -> Result<()> {
        // Create the main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),     // Main content
                Constraint::Length(1),  // Status bar
            ].as_ref())
            .split(f.size());
            
        // Render the main content based on the current view
        match state.get_current_view() {
            View::Accounts => self.render_accounts_view(f, state, chunks[0])?,
            View::Folders => self.render_folders_view(f, state, chunks[0])?,
            View::Emails => self.render_emails_view(f, state, chunks[0])?,
            View::EmailDetail => self.render_email_detail_view(f, state, chunks[0])?,
            View::ComposeEmail => self.render_compose_email_view(f, state, chunks[0])?,
            View::Settings => self.render_settings_view(f, state, chunks[0])?,
            View::AccountConfig => self.render_account_config_view(f, state, chunks[0])?,
        }
        
        // Render the status bar
        self.render_status_bar(f, state, chunks[1])?;
        
        Ok(())
    }
    
    /// Renders the accounts view.
    ///
    /// # Parameters
    /// - `f`: The frame to render on
    /// - `state`: The application state
    /// - `area`: The area to render in
    ///
    /// # Returns
    /// A Result indicating success or failure
    fn render_accounts_view(&self, f: &mut Frame, state: &AppState, area: Rect) -> Result<()> {
        // Get accounts
        let accounts = state.accounts.iter().map(|account| {
            let account_lock = account.blocking_lock();
            account_lock.clone()
        }).collect::<Vec<_>>();
        
        // Render accounts
        views::render_accounts(f, area, &accounts, state.get_selected_account());
        
        Ok(())
    }
    
    /// Renders the folders view.
    ///
    /// # Parameters
    /// - `f`: The frame to render on
    /// - `state`: The application state
    /// - `area`: The area to render in
    ///
    /// # Returns
    /// A Result indicating success or failure
    fn render_folders_view(&self, f: &mut Frame, state: &AppState, area: Rect) -> Result<()> {
        // Get folders
        let folders = if let Some(index) = state.get_selected_account() {
            if index < state.accounts.len() {
                let account = &state.accounts[index];
                let account_lock = account.blocking_lock();
                account_lock.folders.clone()
            } else {
                vec!["INBOX".to_string()]
            }
        } else {
            vec!["INBOX".to_string()]
        };
        
        // Render folders
        views::render_folders(f, area, &folders, Some(0));
        
        Ok(())
    }
    
    /// Renders the emails view.
    ///
    /// # Parameters
    /// - `f`: The frame to render on
    /// - `state`: The application state
    /// - `area`: The area to render in
    ///
    /// # Returns
    /// A Result indicating success or failure
    fn render_emails_view(&self, f: &mut Frame, state: &AppState, area: Rect) -> Result<()> {
        // Render emails
        views::render_emails(f, area, &state.emails, state.get_selected_email());
        
        Ok(())
    }
    
    /// Renders the email detail view.
    ///
    /// # Parameters
    /// - `f`: The frame to render on
    /// - `state`: The application state
    /// - `area`: The area to render in
    ///
    /// # Returns
    /// A Result indicating success or failure
    fn render_email_detail_view(&self, f: &mut Frame, state: &AppState, area: Rect) -> Result<()> {
        // Render email detail
        if let Some(email) = state.get_viewed_email() {
            views::render_email_detail(f, area, email, state.get_email_scroll_offset());
        }
        
        Ok(())
    }
    
    /// Renders the compose email view.
    ///
    /// # Parameters
    /// - `f`: The frame to render on
    /// - `state`: The application state
    /// - `area`: The area to render in
    ///
    /// # Returns
    /// A Result indicating success or failure
    fn render_compose_email_view(&self, f: &mut Frame, _state: &AppState, area: Rect) -> Result<()> {
        // TODO: Implement compose email view
        let compose = Paragraph::new("Compose Email (not implemented)")
            .block(Block::default().borders(Borders::ALL).title("Compose Email"));
            
        f.render_widget(compose, area);
        
        Ok(())
    }
    
    /// Renders the settings view.
    ///
    /// # Parameters
    /// - `f`: The frame to render on
    /// - `state`: The application state
    /// - `area`: The area to render in
    ///
    /// # Returns
    /// A Result indicating success or failure
    fn render_settings_view(&self, f: &mut Frame, _state: &AppState, area: Rect) -> Result<()> {
        // Render settings
        views::render_settings(f, area);
        
        Ok(())
    }
    
    /// Renders the account configuration view.
    ///
    /// # Parameters
    /// - `f`: The frame to render on
    /// - `state`: The application state
    /// - `area`: The area to render in
    ///
    /// # Returns
    /// A Result indicating success or failure
    fn render_account_config_view(&self, f: &mut Frame, state: &AppState, area: Rect) -> Result<()> {
        // Render account configuration
        if let Some(form_state) = state.get_account_form_state() {
            views::account_config::render_account_config(f, area, form_state);
        }
        
        Ok(())
    }
    
    /// Renders the status bar.
    ///
    /// # Parameters
    /// - `f`: The frame to render on
    /// - `state`: The application state
    /// - `area`: The area to render in
    ///
    /// # Returns
    /// A Result indicating success or failure
    fn render_status_bar(&self, f: &mut Frame, state: &AppState, area: Rect) -> Result<()> {
        // Create status message
        let status_message = if let Some(message) = state.get_status_message() {
            message.clone()
        } else {
            match state.get_current_view() {
                View::Accounts => "Accounts - Press 'a' to add, 'e' to edit, 'd' to delete, Enter to select".to_string(),
                View::Folders => "Folders - Press Enter to select, Esc to go back".to_string(),
                View::Emails => "Emails - Press Enter to view, 'c' to compose, 'r' to reply, 'f' to forward, 'd' to delete, Esc to go back".to_string(),
                View::EmailDetail => "Email - Press 'r' to reply, 'f' to forward, 'd' to delete, Esc to go back".to_string(),
                View::ComposeEmail => "Compose - Press Ctrl+s to send, Esc to cancel".to_string(),
                View::Settings => "Settings - Press Esc to go back".to_string(),
                View::AccountConfig => "Account Configuration - Press Enter to edit field, Tab to navigate, Enter on Save to save".to_string(),
            }
        };
        
        // Create status bar
        let status_bar = Paragraph::new(status_message)
            .style(Style::default().fg(Color::White));
        
        f.render_widget(status_bar, area);
        
        Ok(())
    }
}

impl Default for AppRenderer {
    fn default() -> Self {
        Self::new()
    }
}
