use crate::{EmailBuilder, EmailSender, ResendClient};

fn client() -> Option<ResendClient> {
    let key = std::env::var("RESEND_API_KEY").ok()?;
    Some(ResendClient::new(key))
}

fn base_builder() -> EmailBuilder {
    EmailBuilder::new().from("support@sitwego.com").to("sityf237@gmail.com")
}

// ── send html email ───────────────────────────────────────────────────────

#[tokio::test]
async fn send_html_email() {
    let Some(client) = client() else {
        eprintln!("skipping: RESEND_API_KEY not set");
        return;
    };

    let msg = base_builder()
        .subject("Sit We Go — test email (HTML)")
        .html("<h1>It works!</h1><p>This is a test from the <strong>email_api</strong> package.</p>")
        .build()
        .expect("valid message");

    let receipt = client.send(msg).await.expect("send should succeed");
    assert!(!receipt.id.is_empty(), "receipt id should be non-empty");
    println!("email id: {}", receipt.id);
}

// ── send plain-text email ─────────────────────────────────────────────────

#[tokio::test]
async fn send_text_email() {
    let Some(client) = client() else {
        eprintln!("skipping: RESEND_API_KEY not set");
        return;
    };

    let msg = base_builder()
        .subject("Sit We Go — test email (plain text)")
        .text("It works! This is a plain-text test from the email_api package.")
        .build()
        .expect("valid message");

    let receipt = client.send(msg).await.expect("send should succeed");
    assert!(!receipt.id.is_empty(), "receipt id should be non-empty");
    println!("email id: {}", receipt.id);
}

// ── send email with both html and text ────────────────────────────────────

#[tokio::test]
async fn send_html_with_text_fallback() {
    let Some(client) = client() else {
        eprintln!("skipping: RESEND_API_KEY not set");
        return;
    };

    let msg = base_builder()
        .subject("Sit We Go — test email (HTML + text fallback)")
        .html("<h1>Ride booked!</h1><p>Your driver is on the way.</p>")
        .text("Ride booked! Your driver is on the way.")
        .build()
        .expect("valid message");

    let receipt = client.send(msg).await.expect("send should succeed");
    assert!(!receipt.id.is_empty(), "receipt id should be non-empty");
    println!("email id: {}", receipt.id);
}

// ── builder validation ────────────────────────────────────────────────────

#[test]
fn builder_rejects_missing_from() {
    let err = EmailBuilder::new()
        .to("sityf237@gmail.com")
        .subject("test")
        .html("<p>hi</p>")
        .build();
    assert!(err.is_err());
}

#[test]
fn builder_rejects_no_recipients() {
    let err = EmailBuilder::new()
        .from("support@sitwego.com")
        .subject("test")
        .html("<p>hi</p>")
        .build();
    assert!(err.is_err());
}

#[test]
fn builder_rejects_no_body() {
    let err = EmailBuilder::new()
        .from("support@sitwego.com")
        .to("sityf237@gmail.com")
        .subject("test")
        .build();
    assert!(err.is_err());
}
