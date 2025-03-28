//! Email protocol implementations for the Linksy email client.
//! 
//! This module contains implementations for the various email protocols
//! used by the application, including IMAP, POP3, and SMTP.

mod imap;
mod pop3;
mod smtp;

pub use imap::*;
pub use pop3::*;
pub use smtp::*;
