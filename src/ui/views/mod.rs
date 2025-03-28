//! UI views for the Linksy email client.
//! 
//! This module contains the different views used in the application UI.

use crate::models::{Account, Email};
use tui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph, ListState},
    Frame,
};

/// Renders the accounts view.
///
/// # Parameters
/// - `f`: The frame to render on
/// - `area`: The area to render in
/// - `accounts`: The accounts to display
/// - `selected`: The index of the selected account
pub fn render_accounts(
    f: &mut Frame,
    area: Rect,
    accounts: &[Account],
    selected: Option<usize>,
) {
    let account_items: Vec<ListItem> = accounts.iter()
        .map(|account| ListItem::new(account.get_display_name()))
        .collect();
        
    let accounts_list = List::new(account_items)
        .block(Block::default().borders(Borders::ALL).title("Accounts"))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");
        
    let mut state = ListState::default();
    if let Some(i) = selected {
        state.select(Some(i));
    }
    
    f.render_stateful_widget(accounts_list, area, &mut state);
}

/// Renders the folders view.
///
/// # Parameters
/// - `f`: The frame to render on
/// - `area`: The area to render in
/// - `folders`: The folders to display
/// - `selected`: The index of the selected folder
pub fn render_folders(
    f: &mut Frame,
    area: Rect,
    folders: &[String],
    selected: Option<usize>,
) {
    let folder_items: Vec<ListItem> = folders.iter()
        .map(|folder| ListItem::new(folder.clone()))
        .collect();
        
    let folders_list = List::new(folder_items)
        .block(Block::default().borders(Borders::ALL).title("Folders"))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");
        
    let mut state = ListState::default();
    if let Some(i) = selected {
        state.select(Some(i));
    }
    
    f.render_stateful_widget(folders_list, area, &mut state);
}

/// Renders the emails view.
///
/// # Parameters
/// - `f`: The frame to render on
/// - `area`: The area to render in
/// - `emails`: The emails to display
/// - `selected`: The index of the selected email
pub fn render_emails(
    f: &mut Frame,
    area: Rect,
    emails: &[Email],
    selected: Option<usize>,
) {
    let email_items: Vec<ListItem> = emails.iter()
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
        
    let mut state = ListState::default();
    if let Some(i) = selected {
        state.select(Some(i));
    }
    
    f.render_stateful_widget(emails_list, area, &mut state);
}

/// Renders the email detail view.
///
/// # Parameters
/// - `f`: The frame to render on
/// - `area`: The area to render in
/// - `email`: The email to display
pub fn render_email_detail(
    f: &mut Frame,
    area: Rect,
    email: &Email,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(0),     // Body
        ].as_ref())
        .split(area);
        
    // Draw header
    let from = format!("From: {}", if let Some(name) = &email.from_name {
        format!("{} <{}>", name, email.from)
    } else {
        email.from.clone()
    });
    
    let to = format!("To: {}", email.to.join(", "));
    let subject = format!("Subject: {}", email.subject);
    
    let header_text = vec![
        from,
        to,
        subject,
    ].join("\n");
    
    let header = Paragraph::new(header_text)
        .block(Block::default().borders(Borders::ALL).title("Email"));
    
    f.render_widget(header, chunks[0]);
    
    // Draw body
    let body_text = if let Some(text) = &email.body_text {
        text.clone()
    } else if let Some(html) = &email.body_html {
        // TODO: Implement HTML to text conversion
        format!("HTML email: {}", html)
    } else {
        "No content".to_string()
    };
    
    let body = Paragraph::new(body_text)
        .block(Block::default().borders(Borders::ALL).title("Body"));
        
    f.render_widget(body, chunks[1]);
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
pub fn render_compose_email(
    f: &mut Frame,
    area: Rect,
    to: &str,
    subject: &str,
    body: &str,
    cursor_position: (u16, u16),
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(0),     // Body
        ].as_ref())
        .split(area);
        
    // Draw header
    let to_line = format!("To: {}", to);
    let subject_line = format!("Subject: {}", subject);
    
    let header_text = vec![
        to_line,
        subject_line,
    ].join("\n");
    
    let header = Paragraph::new(header_text)
        .block(Block::default().borders(Borders::ALL).title("Compose Email"));
    
    f.render_widget(header, chunks[0]);
    
    // Draw body
    let body_widget = Paragraph::new(body)
        .block(Block::default().borders(Borders::ALL).title("Body"));
        
    f.render_widget(body_widget, chunks[1]);
    
    // Set cursor position
    f.set_cursor(cursor_position.0, cursor_position.1);
}

/// Renders the settings view.
///
/// # Parameters
/// - `f`: The frame to render on
/// - `area`: The area to render in
pub fn render_settings(
    f: &mut Frame,
    area: Rect,
) {
    let settings = Paragraph::new("Settings (not implemented)")
        .block(Block::default().borders(Borders::ALL).title("Settings"));
        
    f.render_widget(settings, area);
}
