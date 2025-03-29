# Linksy

A light and fast terminal-based email client written in Rust.

![Linksy Email Client](https://via.placeholder.com/800x450.png?text=Linksy+Email+Client)

## Overview

Linksy is a modern, efficient email client designed for the terminal. It provides a clean, intuitive interface for managing multiple email accounts, reading and composing emails, and organizing your inbox—all without leaving your terminal.

## Features

- **Terminal-based UI**: Clean, responsive interface using the Ratatui library
- **Multiple Protocol Support**:
  - IMAP for synchronizing with mail servers
  - POP3 for retrieving emails
  - SMTP for sending emails
- **Account Management**: Configure and use multiple email accounts
- **Folder Navigation**: Browse through your email folders
- **Email Operations**:
  - View emails with proper formatting
  - Compose new emails
  - Reply to and forward emails
  - Mark emails as read/unread
  - Flag important emails
  - Delete unwanted emails
- **Local Storage**: Emails are cached locally for offline access
- **Keyboard-driven Interface**: Efficient navigation using keyboard shortcuts

## Installation

### Prerequisites

- Rust and Cargo (1.75.0 or newer)

### From Source

```bash
# Clone the repository
git clone https://github.com/yourusername/linksy.git
cd linksy

# Build the project
cargo build --release

# Run the application
cargo run --release
```

### From Cargo

```bash
cargo install linksy
```

## Usage

### Starting Linksy

```bash
linksy
```

### Keyboard Shortcuts

#### Global

- `Ctrl+q`: Quit application

#### Account View

- `↑/↓`: Navigate between accounts
- `Enter`: Select account and view folders
- `a`: Add new account
- `d`: Delete selected account

#### Folder View

- `↑/↓`: Navigate between folders
- `Enter`: Select folder and view emails
- `Esc`: Go back to accounts view

#### Email List View

- `↑/↓`: Navigate between emails
- `Enter`: View selected email
- `c`: Compose new email
- `r`: Reply to selected email
- `f`: Forward selected email
- `d`: Delete selected email
- `Esc`: Go back to folders view

#### Email Detail View

- `↑/↓`: Scroll email content
- `PgUp/PgDn`: Scroll by page
- `Home/End`: Jump to top/bottom
- `r`: Reply to email
- `f`: Forward email
- `d`: Delete email
- `Esc`: Go back to email list

#### Compose View

- `Ctrl+s`: Send email
- `Esc`: Cancel and go back

## Configuration

Linksy stores its configuration in:

```
~/.config/linksy/config.json
```

### Example Configuration

```json
{
  "accounts": [
    {
      "id": "personal",
      "name": "Personal Email",
      "email": "your.email@example.com",
      "imap": {
        "host": "imap.example.com",
        "port": 993,
        "username": "your.email@example.com",
        "password": "your_password",
        "use_ssl": true
      },
      "smtp": {
        "host": "smtp.example.com",
        "port": 587,
        "username": "your.email@example.com",
        "password": "your_password",
        "use_starttls": true
      }
    }
  ],
  "settings": {
    "check_interval": 300,
    "notifications": true,
    "theme": "default"
  }
}
```

## Development

### Project Structure

- `src/main.rs`: Application entry point
- `src/config/`: Configuration management
- `src/models/`: Data models for accounts, emails, etc.
- `src/protocols/`: Email protocol implementations (IMAP, POP3, SMTP)
- `src/storage/`: Local email storage
- `src/ui/`: Terminal UI implementation

### Building and Testing

```bash
# Build the project
cargo build

# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## Acknowledgments

- [Ratatui](https://github.com/ratatui-org/ratatui) for the terminal UI framework
- [Tokio](https://tokio.rs/) for the async runtime
- [Lettre](https://github.com/lettre/lettre) for SMTP support
- [IMAP crate](https://github.com/jonhoo/rust-imap) for IMAP support
