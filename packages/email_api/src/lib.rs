pub mod builder;
pub mod client;
#[cfg(test)]
mod tests;
pub mod types;

pub use builder::{BuildError, EmailBuilder};
pub use client::{EmailError, EmailReceipt, EmailSender, ResendClient};
pub use types::EmailMessage;
