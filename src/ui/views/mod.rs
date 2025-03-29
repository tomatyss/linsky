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
#[allow(dead_code)]
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
#[allow(dead_code)]
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
#[allow(dead_code)]
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
/// - `scroll_offset`: The vertical scroll offset for the email body
pub fn render_email_detail(
    f: &mut Frame,
    area: Rect,
    email: &Email,
    scroll_offset: u16,
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
    
    // Process and draw body
    let body_text = if let Some(text) = &email.body_text {
        format_plain_text(text)
    } else if let Some(html) = &email.body_html {
        convert_html_to_text(html)
    } else {
        "No content".to_string()
    };
    
    // Create a scrollable paragraph for the body
    let body = Paragraph::new(body_text.clone())
        .block(Block::default().borders(Borders::ALL).title("Body"))
        .scroll((scroll_offset, 0))
        .wrap(tui::widgets::Wrap { trim: false });
        
    f.render_widget(body, chunks[1]);
    
    // Draw scroll indicator if needed
    let body_height = chunks[1].height as usize - 2; // Account for borders
    let lines: Vec<&str> = body_text.lines().collect();
    
    if lines.len() > body_height {
        let scroll_indicator = format!("(Scroll: {}/{})", 
            scroll_offset.saturating_add(1), 
            lines.len().saturating_sub(body_height).saturating_add(1)
        );
        
        let scroll_text = Paragraph::new(scroll_indicator.clone())
            .style(Style::default().fg(Color::Gray));
            
        let scroll_area = Rect::new(
            chunks[1].x + chunks[1].width - scroll_indicator.len() as u16 - 2,
            chunks[1].y + chunks[1].height - 1,
            scroll_indicator.len() as u16,
            1
        );
        
        f.render_widget(scroll_text, scroll_area);
    }
}

/// Formats plain text for better display.
///
/// # Parameters
/// - `text`: The plain text to format
///
/// # Returns
/// Formatted text with proper line breaks and spacing
fn format_plain_text(text: &str) -> String {
    // Normalize line endings
    let text = text.replace("\r\n", "\n");
    
    // Remove excessive blank lines (more than 2 consecutive)
    let mut result = String::new();
    let mut blank_line_count = 0;
    
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            blank_line_count += 1;
            if blank_line_count <= 2 {
                result.push_str("\n");
            }
        } else {
            blank_line_count = 0;
            result.push_str(line);
            result.push('\n');
        }
    }
    
    result
}

/// Converts HTML to plain text.
///
/// # Parameters
/// - `html`: The HTML content to convert
///
/// # Returns
/// Plain text representation of the HTML content
fn convert_html_to_text(html: &str) -> String {
    // This is a basic implementation - a more robust solution would use a proper HTML parser
    
    // Replace common HTML entities
    let text = html
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'");
    
    // Replace common block elements with line breaks
    let text = text
        .replace("<br>", "\n")
        .replace("<br/>", "\n")
        .replace("<br />", "\n")
        .replace("<p>", "\n")
        .replace("</p>", "\n")
        .replace("<div>", "\n")
        .replace("</div>", "\n")
        .replace("<tr>", "\n")
        .replace("</tr>", "\n")
        .replace("<li>", "\n- ")
        .replace("</li>", "");
    
    // Remove all other HTML tags
    let mut result = String::new();
    let mut in_tag = false;
    
    for c in text.chars() {
        if c == '<' {
            in_tag = true;
        } else if c == '>' {
            in_tag = false;
        } else if !in_tag {
            result.push(c);
        }
    }
    
    // Normalize whitespace
    let mut formatted = String::new();
    let mut last_was_whitespace = false;
    
    for c in result.chars() {
        if c.is_whitespace() {
            if !last_was_whitespace || c == '\n' {
                formatted.push(c);
            }
            last_was_whitespace = true;
        } else {
            formatted.push(c);
            last_was_whitespace = false;
        }
    }
    
    // Remove excessive blank lines
    let mut final_result = String::new();
    let mut blank_line_count = 0;
    
    for line in formatted.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            blank_line_count += 1;
            if blank_line_count <= 2 {
                final_result.push_str("\n");
            }
        } else {
            blank_line_count = 0;
            final_result.push_str(trimmed);
            final_result.push('\n');
        }
    }
    
    final_result
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
#[allow(dead_code)]
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
#[allow(dead_code)]
pub fn render_settings(
    f: &mut Frame,
    area: Rect,
) {
    let settings = Paragraph::new("Settings (not implemented)")
        .block(Block::default().borders(Borders::ALL).title("Settings"));
        
    f.render_widget(settings, area);
}
