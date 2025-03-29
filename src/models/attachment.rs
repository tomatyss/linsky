//! Email attachment model for the Linksy email client.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Represents an email attachment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    /// Unique identifier for the attachment
    pub id: String,
    /// Filename of the attachment
    pub filename: String,
    /// MIME content type of the attachment
    pub content_type: String,
    /// Size of the attachment in bytes
    pub size: usize,
    /// Binary data of the attachment
    #[serde(skip)]
    #[allow(dead_code)]
    pub data: Vec<u8>,
}

impl Attachment {
    /// Creates a new attachment from a file.
    ///
    /// # Parameters
    /// - `path`: Path to the file
    ///
    /// # Returns
    /// A Result containing the Attachment or an error
    #[allow(dead_code)]
    pub fn from_file(path: &Path) -> anyhow::Result<Self> {
        let data = std::fs::read(path)?;
        let filename = path.file_name()
            .ok_or_else(|| anyhow::anyhow!("Invalid filename"))?
            .to_string_lossy()
            .to_string();
            
        // Try to determine content type from extension
        let content_type = match path.extension().and_then(|ext| ext.to_str()) {
            Some("pdf") => "application/pdf",
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("png") => "image/png",
            Some("gif") => "image/gif",
            Some("txt") => "text/plain",
            Some("html") | Some("htm") => "text/html",
            Some("doc") => "application/msword",
            Some("docx") => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            Some("xls") => "application/vnd.ms-excel",
            Some("xlsx") => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            Some("zip") => "application/zip",
            _ => "application/octet-stream",
        }.to_string();
        
        Ok(Self {
            id: uuid::Uuid::new_v4().to_string(),
            filename,
            content_type,
            size: data.len(),
            data,
        })
    }
    
    /// Saves the attachment to a file.
    ///
    /// # Parameters
    /// - `path`: Path where to save the attachment
    ///
    /// # Returns
    /// A Result indicating success or failure
    #[allow(dead_code)]
    pub fn save_to_file(&self, path: &Path) -> anyhow::Result<()> {
        std::fs::write(path, &self.data)?;
        Ok(())
    }
    
    /// Gets a human-readable size string.
    ///
    /// # Returns
    /// A string representing the attachment size in a human-readable format
    #[allow(dead_code)]
    pub fn get_size_string(&self) -> String {
        if self.size < 1024 {
            format!("{} B", self.size)
        } else if self.size < 1024 * 1024 {
            format!("{:.1} KB", self.size as f64 / 1024.0)
        } else if self.size < 1024 * 1024 * 1024 {
            format!("{:.1} MB", self.size as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.1} GB", self.size as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    }
}
