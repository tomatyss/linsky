//! User interface implementation for the Linksy email client.
//! 
//! This module contains the terminal-based UI implementation using the tui crate.

pub mod app;
pub mod renderer;
pub mod views;
pub mod widgets;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{io, time::Duration};
use tui::{
    backend::CrosstermBackend,
    Terminal,
};

/// Initializes the terminal for the UI.
///
/// # Returns
/// A Result containing the Terminal or an error
pub fn init_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    // Set up terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    
    Ok(terminal)
}

/// Restores the terminal to its original state.
///
/// # Parameters
/// - `terminal`: The terminal to restore
///
/// # Returns
/// A Result indicating success or failure
pub fn restore_terminal(mut terminal: Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    
    Ok(())
}

/// Waits for a key press event.
///
/// # Parameters
/// - `timeout`: Optional timeout duration
///
/// # Returns
/// A Result containing the KeyEvent or None if timeout occurred
pub fn wait_for_key(timeout: Option<Duration>) -> Result<Option<KeyEvent>> {
    if let Some(duration) = timeout {
        if event::poll(duration)? {
            if let Event::Key(key) = event::read()? {
                return Ok(Some(key));
            }
        }
        Ok(None)
    } else {
        loop {
            if let Event::Key(key) = event::read()? {
                return Ok(Some(key));
            }
        }
    }
}

/// Checks if a key combination was pressed.
///
/// # Parameters
/// - `key`: The key event to check
/// - `code`: The key code to match
/// - `modifiers`: The key modifiers to match
///
/// # Returns
/// true if the key combination matches, false otherwise
pub fn is_key_with_modifier(key: &KeyEvent, code: KeyCode, modifiers: KeyModifiers) -> bool {
    key.code == code && key.modifiers == modifiers
}

/// Checks if a key was pressed without modifiers.
///
/// # Parameters
/// - `key`: The key event to check
/// - `code`: The key code to match
///
/// # Returns
/// true if the key matches and has no modifiers, false otherwise
#[allow(dead_code)]
pub fn is_key(key: &KeyEvent, code: KeyCode) -> bool {
    key.code == code && key.modifiers == KeyModifiers::NONE
}
