//! Custom widgets for the Linksy email client UI.
//! 
//! This module contains custom widgets used in the application UI.

use tui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    widgets::{Block, Paragraph, Widget},
};

/// A widget for displaying an email message.
pub struct EmailView<'a> {
    /// The email subject
    subject: &'a str,
    /// The email sender
    from: &'a str,
    /// The email recipients
    to: &'a str,
    /// The email body
    body: &'a str,
    /// The widget block
    block: Option<Block<'a>>,
    /// The widget style
    style: Style,
}

impl<'a> EmailView<'a> {
    /// Creates a new EmailView.
    ///
    /// # Parameters
    /// - `subject`: The email subject
    /// - `from`: The email sender
    /// - `to`: The email recipients
    /// - `body`: The email body
    ///
    /// # Returns
    /// A new EmailView instance
    pub fn new(subject: &'a str, from: &'a str, to: &'a str, body: &'a str) -> Self {
        Self {
            subject,
            from,
            to,
            body,
            block: None,
            style: Style::default(),
        }
    }
    
    /// Sets the block for the widget.
    ///
    /// # Parameters
    /// - `block`: The block to set
    ///
    /// # Returns
    /// The modified EmailView
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
    
    /// Sets the style for the widget.
    ///
    /// # Parameters
    /// - `style`: The style to set
    ///
    /// # Returns
    /// The modified EmailView
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl<'a> Widget for EmailView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let area = match self.block {
            Some(b) => {
                let inner_area = b.inner(area);
                b.render(area, buf);
                inner_area
            }
            None => area,
        };
        
        if area.height < 4 {
            return;
        }
        
        // Create layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Header
                Constraint::Min(0),     // Body
            ].as_ref())
            .split(area);
            
        // Render header
        let header_text = vec![
            format!("From: {}", self.from),
            format!("To: {}", self.to),
            format!("Subject: {}", self.subject),
        ].join("\n");
        
        let header = Paragraph::new(header_text)
            .style(self.style);
            
        header.render(chunks[0], buf);
        
        // Render body
        let body = Paragraph::new(self.body)
            .style(self.style);
            
        body.render(chunks[1], buf);
    }
}

/// A widget for displaying account information.
pub struct AccountInfoWidget<'a> {
    /// The account name
    name: &'a str,
    /// The account email
    email: &'a str,
    /// The account status
    status: &'a str,
    /// The widget block
    block: Option<Block<'a>>,
    /// The widget style
    style: Style,
}

impl<'a> AccountInfoWidget<'a> {
    /// Creates a new AccountInfoWidget.
    ///
    /// # Parameters
    /// - `name`: The account name
    /// - `email`: The account email
    /// - `status`: The account status
    ///
    /// # Returns
    /// A new AccountInfoWidget instance
    pub fn new(name: &'a str, email: &'a str, status: &'a str) -> Self {
        Self {
            name,
            email,
            status,
            block: None,
            style: Style::default(),
        }
    }
    
    /// Sets the block for the widget.
    ///
    /// # Parameters
    /// - `block`: The block to set
    ///
    /// # Returns
    /// The modified AccountInfoWidget
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
    
    /// Sets the style for the widget.
    ///
    /// # Parameters
    /// - `style`: The style to set
    ///
    /// # Returns
    /// The modified AccountInfoWidget
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl<'a> Widget for AccountInfoWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let area = match self.block {
            Some(b) => {
                let inner_area = b.inner(area);
                b.render(area, buf);
                inner_area
            }
            None => area,
        };
        
        if area.height < 3 {
            return;
        }
        
        // Render account info
        let text = vec![
            format!("Name: {}", self.name),
            format!("Email: {}", self.email),
            format!("Status: {}", self.status),
        ].join("\n");
        
        let paragraph = Paragraph::new(text)
            .style(self.style);
            
        paragraph.render(area, buf);
    }
}

/// A widget for displaying a status bar.
pub struct StatusBar<'a> {
    /// The status message
    message: &'a str,
    /// The widget block
    block: Option<Block<'a>>,
    /// The widget style
    style: Style,
}

impl<'a> StatusBar<'a> {
    /// Creates a new StatusBar.
    ///
    /// # Parameters
    /// - `message`: The status message
    ///
    /// # Returns
    /// A new StatusBar instance
    pub fn new(message: &'a str) -> Self {
        Self {
            message,
            block: None,
            style: Style::default(),
        }
    }
    
    /// Sets the block for the widget.
    ///
    /// # Parameters
    /// - `block`: The block to set
    ///
    /// # Returns
    /// The modified StatusBar
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
    
    /// Sets the style for the widget.
    ///
    /// # Parameters
    /// - `style`: The style to set
    ///
    /// # Returns
    /// The modified StatusBar
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl<'a> Widget for StatusBar<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let area = match self.block {
            Some(b) => {
                let inner_area = b.inner(area);
                b.render(area, buf);
                inner_area
            }
            None => area,
        };
        
        if area.height < 1 {
            return;
        }
        
        // Render status message
        let paragraph = Paragraph::new(self.message)
            .style(self.style);
            
        paragraph.render(area, buf);
    }
}

/// A widget for displaying a text input field.
pub struct InputField<'a> {
    /// The input label
    label: &'a str,
    /// The input value
    value: &'a str,
    /// The widget block
    block: Option<Block<'a>>,
    /// The widget style
    style: Style,
}

impl<'a> InputField<'a> {
    /// Creates a new InputField.
    ///
    /// # Parameters
    /// - `label`: The input label
    /// - `value`: The input value
    ///
    /// # Returns
    /// A new InputField instance
    pub fn new(label: &'a str, value: &'a str) -> Self {
        Self {
            label,
            value,
            block: None,
            style: Style::default(),
        }
    }
    
    /// Sets the block for the widget.
    ///
    /// # Parameters
    /// - `block`: The block to set
    ///
    /// # Returns
    /// The modified InputField
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
    
    /// Sets the style for the widget.
    ///
    /// # Parameters
    /// - `style`: The style to set
    ///
    /// # Returns
    /// The modified InputField
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl<'a> Widget for InputField<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let area = match self.block {
            Some(b) => {
                let inner_area = b.inner(area);
                b.render(area, buf);
                inner_area
            }
            None => area,
        };
        
        if area.height < 1 {
            return;
        }
        
        // Render input field
        let text = format!("{}: {}", self.label, self.value);
        let paragraph = Paragraph::new(text)
            .style(self.style);
            
        paragraph.render(area, buf);
    }
}
