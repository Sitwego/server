pub mod builder;
pub mod client;
pub mod render;
pub mod system;
#[cfg(test)]
mod tests;
pub mod types;

pub use builder::{BuildError, EmailBuilder};
pub use client::{EmailError, EmailReceipt, EmailSender, ResendClient};
pub use render::{RenderError, TemplateRenderer};
pub use system::OtpEmail;
pub use types::{EmailMessage, RenderedEmail, StoredTemplate};
