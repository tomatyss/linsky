//! Data models for the Linksy email client.
//! 
//! This module contains the core data structures used throughout the application,
//! including representations of emails, attachments, and accounts.

mod email;
mod account;
mod attachment;

pub use email::*;
pub use account::*;
