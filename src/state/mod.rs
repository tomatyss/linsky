//! Application state management for the Linksy email client.
//! 
//! This module contains components for managing the application state,
//! including accounts, emails, and UI state.

mod app_state;
mod account_manager;
mod email_manager;

pub use app_state::*;
pub use account_manager::*;
pub use email_manager::*;
