use crate::{
    EmailBuilder, EmailSender, OtpEmail, ResendClient, StoredTemplate,
    TemplateRenderer,
};

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

// ── template rendering (admin / Unlayer path) ──────────────────────────────

fn stored(subject_src: &str, html_src: &str) -> StoredTemplate {
    StoredTemplate {
        slug: "test_tpl".into(),
        subject_src: subject_src.into(),
        html_src: html_src.into(),
        from: None,
        reply_to: None,
    }
}

#[test]
fn renders_subject_and_body_with_context() {
    let tpl = stored(
        "Ride to {{ destination }}",
        "<p>Hi {{ rider_name }}, fare {{ fare }}.</p>",
    );
    let ctx = serde_json::json!({
        "destination": "Airport",
        "rider_name": "Alex",
        "fare": "12.50",
    });

    let out = TemplateRenderer::new().render(&tpl, &ctx).unwrap();
    assert_eq!(out.subject, "Ride to Airport");
    assert_eq!(out.html, "<p>Hi Alex, fare 12.50.</p>");
}

#[test]
fn escapes_variable_values_but_not_template_markup() {
    // The `<b>` is the template's own markup (passes through); the value's `<`
    // must be escaped so injected data cannot break or inject markup.
    let tpl = stored("s", "<b>{{ note }}</b>");
    let ctx = serde_json::json!({ "note": "<script>x</script>" });

    let out = TemplateRenderer::new().render(&tpl, &ctx).unwrap();
    // minijinja HTML-escapes `<`, `>` and also `/` (as `&#x2f;`).
    assert_eq!(out.html, "<b>&lt;script&gt;x&lt;&#x2f;script&gt;</b>");
    // The template's own <b> markup is untouched.
    assert!(out.html.starts_with("<b>") && out.html.ends_with("</b>"));
}

#[test]
fn strict_mode_errors_on_undeclared_variable() {
    let tpl = stored("s", "<p>{{ missing_var }}</p>");
    let ctx = serde_json::json!({});

    assert!(TemplateRenderer::new().render(&tpl, &ctx).is_err());
}

// ── system emails (askama path) ────────────────────────────────────────────

#[test]
fn otp_system_email_renders() {
    let out = OtpEmail { code: "482913".into(), expiry_minutes: 10 }
        .render_email()
        .unwrap();
    assert_eq!(out.subject, OtpEmail::SUBJECT);
    assert!(out.html.contains("482913"));
    assert!(out.html.contains("10 minutes"));
}
