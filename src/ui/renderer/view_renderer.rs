//! View renderer for the Linksy email client.

use crate::models::{AccountSummary, Email};
use crate::ui::views;
use anyhow::Result;
use tui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Renders specific views in the application.
pub struct ViewRenderer;

impl ViewRenderer {
    /// Creates a new ViewRenderer instance.
    ///
    /// # Returns
    /// A new ViewRenderer instance
    pub fn new() -> Self {
        Self
    }
    
    /// Renders the accounts view.
    ///
    /// # Parameters
    /// - `f`: The frame to render on
    /// - `area`: The area to render in
    /// - `accounts`: The accounts to display
    /// - `selected`: The index of the selected account
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub fn render_accounts(
        &self,
        f: &mut Frame,
        area: Rect,
        accounts: &[AccountSummary],
        selected: Option<usize>,
    ) -> Result<()> {
        views::render_accounts(f, area, accounts, selected);
        
        Ok(())
    }
    
    /// Renders the folders view.
    ///
    /// # Parameters
    /// - `f`: The frame to render on
    /// - `area`: The area to render in
    /// - `folders`: The folders to display
    /// - `selected`: The index of the selected folder
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub fn render_folders(
        &self,
        f: &mut Frame,
        area: Rect,
        folders: &[String],
        selected: Option<usize>,
    ) -> Result<()> {
        views::render_folders(f, area, folders, selected);
        
        Ok(())
    }
    
    /// Renders the emails view.
    ///
    /// # Parameters
    /// - `f`: The frame to render on
    /// - `area`: The area to render in
    /// - `emails`: The emails to display
    /// - `selected`: The index of the selected email
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub fn render_emails(
        &self,
        f: &mut Frame,
        area: Rect,
        emails: &[Email],
        selected: Option<usize>,
    ) -> Result<()> {
        views::render_emails(f, area, emails, selected);
        
        Ok(())
    }
    
    /// Renders the email detail view.
    ///
    /// # Parameters
    /// - `f`: The frame to render on
    /// - `area`: The area to render in
    /// - `email`: The email to display
    /// - `scroll_offset`: The vertical scroll offset for the email body
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub fn render_email_detail(
        &self,
        f: &mut Frame,
        area: Rect,
        email: &Email,
        scroll_offset: u16,
    ) -> Result<()> {
        views::render_email_detail(f, area, email, scroll_offset);
        
        Ok(())
    }
    
    /// Renders the compose email view.
    ///
    /// # Parameters
    /// - `f`: The frame to render on
    /// - `area`: The area to render in
    /// - `to`: The recipient
    /// - `subject`: The subject
    /// - `body`: The body
    /// - `cursor_position`: The cursor position
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub fn render_compose_email(
        &self,
        f: &mut Frame,
        area: Rect,
        to: &str,
        subject: &str,
        body: &str,
        cursor_position: (u16, u16),
    ) -> Result<()> {
        views::render_compose_email(f, area, to, subject, body, cursor_position);
        
        Ok(())
    }
    
    /// Renders the settings view.
    ///
    /// # Parameters
    /// - `f`: The frame to render on
    /// - `area`: The area to render in
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub fn render_settings(
        &self,
        f: &mut Frame,
        area: Rect,
    ) -> Result<()> {
        views::render_settings(f, area);
        
        Ok(())
    }
    
    /// Renders the account configuration view.
    ///
    /// # Parameters
    /// - `f`: The frame to render on
    /// - `area`: The area to render in
    /// - `form_state`: The account form state
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub fn render_account_config(
        &self,
        f: &mut Frame,
        area: Rect,
        form_state: &crate::ui::views::account_config::AccountFormState,
    ) -> Result<()> {
        views::account_config::render_account_config(f, area, form_state);
        
        Ok(())
    }
    
    /// Renders a message.
    ///
    /// # Parameters
    /// - `f`: The frame to render on
    /// - `area`: The area to render in
    /// - `title`: The title of the message
    /// - `message`: The message to display
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub fn render_message(
        &self,
        f: &mut Frame,
        area: Rect,
        title: &str,
        message: &str,
    ) -> Result<()> {
        let message_widget = Paragraph::new(message)
            .block(Block::default().borders(Borders::ALL).title(title));
            
        f.render_widget(message_widget, area);
        
        Ok(())
    }
    
    /// Renders a loading indicator.
    ///
    /// # Parameters
    /// - `f`: The frame to render on
    /// - `area`: The area to render in
    /// - `message`: The message to display
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub fn render_loading(
        &self,
        f: &mut Frame,
        area: Rect,
        message: &str,
    ) -> Result<()> {
        let loading_widget = Paragraph::new(message)
            .block(Block::default().borders(Borders::ALL).title("Loading"));
            
        f.render_widget(loading_widget, area);
        
        Ok(())
    }
    
    /// Renders a split view with two panels.
    ///
    /// # Parameters
    /// - `f`: The frame to render on
    /// - `area`: The area to render in
    /// - `left_title`: The title of the left panel
    /// - `right_title`: The title of the right panel
    /// - `left_content`: The content of the left panel
    /// - `right_content`: The content of the right panel
    /// - `split_ratio`: The ratio of the split (0.0 - 1.0)
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub fn render_split_view(
        &self,
        f: &mut Frame,
        area: Rect,
        left_title: &str,
        right_title: &str,
        left_content: &str,
        right_content: &str,
        split_ratio: f32,
    ) -> Result<()> {
        // Calculate constraints
        let left_size = (area.width as f32 * split_ratio) as u16;
        let right_size = area.width - left_size;
        
        // Create layout
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(left_size),
                Constraint::Length(right_size),
            ].as_ref())
            .split(area);
            
        // Create widgets
        let left_widget = Paragraph::new(left_content)
            .block(Block::default().borders(Borders::ALL).title(left_title));
            
        let right_widget = Paragraph::new(right_content)
            .block(Block::default().borders(Borders::ALL).title(right_title));
            
        // Render widgets
        f.render_widget(left_widget, chunks[0]);
        f.render_widget(right_widget, chunks[1]);
        
        Ok(())
    }
}

impl Default for ViewRenderer {
    fn default() -> Self {
        Self::new()
    }
}
