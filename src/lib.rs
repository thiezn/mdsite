//! Minimal static site generator from Markdown.
//!
//! Library API: [`build`].

mod css;
mod error;
mod generate;
mod html;
mod markdown;
mod mermaid;
mod walk;

pub use error::{Error, Result};
pub use generate::build;
