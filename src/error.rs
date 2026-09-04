//! Error types for the mdsite library.

use std::io;
use std::path::PathBuf;
use std::string::FromUtf8Error;

/// Library error type. Callers can use `?` freely with I/O and walk operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("invalid UTF-8: {0}")]
    Utf8(#[from] FromUtf8Error),

    #[error("walkdir error: {0}")]
    WalkDir(#[from] walkdir::Error),

    #[error("input path is not a directory: {0}")]
    InputNotDirectory(PathBuf),

    #[error("mermaid-cli (`mmdc`) was not found on PATH. Install @mermaid-js/mermaid-cli (mmdc) to render Mermaid diagrams.")]
    MermaidCliMissing,

    #[error("mermaid-cli failed for {path}: {message}")]
    MermaidRender {
        path: PathBuf,
        message: String,
    },

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, Error>;
