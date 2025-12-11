//! Email sender using Gmail SMTP.

use anyhow::{Context, Result};
use lettre::message::{header::ContentType, Mailbox, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

use super::config::DigestConfig;

/// Email sender for research digests.
pub struct EmailSender {
    config: DigestConfig,
}

impl EmailSender {
    /// Create a new email sender with the given configuration.
    #[must_use]
    pub const fn new(config: DigestConfig) -> Self {
        Self { config }
    }

    /// Create from environment variables.
    pub fn from_env() -> Result<Self> {
        let config = DigestConfig::from_env()?;
        Ok(Self::new(config))
    }

    /// Send an email with HTML and plain-text content.
    pub async fn send(&self, subject: &str, html_body: &str, text_body: &str) -> Result<()> {
        let from: Mailbox = self
            .config
            .from_email
            .parse()
            .context("Invalid from email address")?;

        let to: Mailbox = self
            .config
            .to_email
            .parse()
            .context("Invalid to email address")?;

        // Build multipart message with both HTML and plain text
        let email = Message::builder()
            .from(from)
            .to(to)
            .subject(subject)
            .multipart(
                MultiPart::alternative()
                    .singlepart(
                        SinglePart::builder()
                            .header(ContentType::TEXT_PLAIN)
                            .body(text_body.to_string()),
                    )
                    .singlepart(
                        SinglePart::builder()
                            .header(ContentType::TEXT_HTML)
                            .body(html_body.to_string()),
                    ),
            )
            .context("Failed to build email message")?;

        // Create SMTP transport with STARTTLS
        let creds = Credentials::new(
            self.config.smtp_username.clone(),
            self.config.smtp_password.clone(),
        );

        let mailer: AsyncSmtpTransport<Tokio1Executor> =
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&self.config.smtp_host)
                .context("Failed to create SMTP transport")?
                .port(self.config.smtp_port)
                .credentials(creds)
                .build();

        // Send the email
        mailer
            .send(email)
            .await
            .context("Failed to send email via SMTP")?;

        tracing::info!(
            to = %self.config.to_email,
            subject = subject,
            "Email sent successfully"
        );

        Ok(())
    }

    /// Send a simple test email to verify configuration.
    pub async fn send_test(&self) -> Result<()> {
        let subject = "CTO Research Digest - Test Email";
        let html_body = r#"
<!DOCTYPE html>
<html>
<head>
    <style>
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; padding: 20px; }
        .container { max-width: 600px; margin: 0 auto; }
        h1 { color: #2563eb; }
        .success { color: #16a34a; font-weight: bold; }
    </style>
</head>
<body>
    <div class="container">
        <h1>🔬 CTO Research Digest</h1>
        <p class="success">✅ Email configuration is working!</p>
        <p>This is a test email from the CTO Research pipeline.</p>
        <p>If you're seeing this, Gmail SMTP is configured correctly.</p>
        <hr>
        <p style="color: #6b7280; font-size: 12px;">
            Sent from CTO Research Pipeline
        </p>
    </div>
</body>
</html>
"#;

        let text_body = r"
CTO Research Digest - Test Email

✅ Email configuration is working!

This is a test email from the CTO Research pipeline.
If you're seeing this, Gmail SMTP is configured correctly.

---
Sent from CTO Research Pipeline
";

        self.send(subject, html_body, text_body).await
    }
}




