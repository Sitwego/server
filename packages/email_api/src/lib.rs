pub mod builder;
pub mod client;
pub mod types;
#[cfg(test)]
mod tests;

pub use builder::{BuildError, EmailBuilder};
pub use client::{EmailError, EmailReceipt, EmailSender, ResendClient};
pub use types::EmailMessage;
