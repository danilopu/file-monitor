use lettre::{Message, SmtpTransport, Transport};
// use lettre::message::Mailbox;
use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct EmailSettings {
    pub smtp_server: String,
    pub smtp_user: String,
    pub smtp_password: String,
    pub sender: String,
    pub recipient: String,
}

pub fn send_email(settings: &EmailSettings, body: String) -> Result<(), Box<dyn std::error::Error>> {
    let email = Message::builder()
        .from(settings.sender.parse()?)
        .to(settings.recipient.parse()?)
        .subject("PDF File Update Notification")
        .body(body)?;

    let mailer = SmtpTransport::relay(&settings.smtp_server)?
        .credentials(lettre::transport::smtp::authentication::Credentials::new(
            settings.smtp_user.clone(),
            settings.smtp_password.clone(),
        ))
        .build();

    mailer.send(&email)?;
    Ok(())
}