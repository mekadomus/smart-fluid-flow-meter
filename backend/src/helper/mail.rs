use crate::{
    api::{fluid_meter::FluidMeter, user::User},
    settings::settings::Settings,
};

use async_trait::async_trait;
use mockall::automock;
use reqwest::Client;
use tracing::error;

#[automock]
#[async_trait]
pub trait MailHelper: Send + Sync {
    async fn constant_flow_alert(
        &self,
        settings: &Settings,
        user: &User,
        meter: &FluidMeter,
    ) -> bool;
    async fn password_recovery(&self, settings: &Settings, user: &User, token: &str) -> bool;
    async fn sign_up_verification(&self, settings: &Settings, user: &User, token: &str) -> bool;
}

pub struct DefaultMailHelper;

#[async_trait]
impl MailHelper for DefaultMailHelper {
    /// Sends user an e-mail to verify they signed up to the system
    async fn sign_up_verification(&self, settings: &Settings, user: &User, token: &str) -> bool {
        let body_str = format!(
            r#"{{
            "sender": {{
                "name": "{}",
                "email": "{}"
            }},
            "to": [
                {{
                    "name": "{}",
                    "email": "{}"
                }}
            ],
            "subject": "Welcome to Mekadomus",
            "htmlContent": "<html><body>Hello {},<br />To complete your sign-up, click the following link.<br /><br /><a href=\"https://console.mekadomus.com/email-verification/{}\">Verify Email</a><br><br></body></html>"
        }}"#,
            settings.mail.mailer_name,
            settings.mail.mailer_address,
            user.name,
            user.email,
            user.name,
            token
        );
        let body: serde_json::Value = match serde_json::from_str(&body_str) {
            Ok(b) => b,
            Err(e) => {
                error!("Couldn't parse json. {}", e);
                return false;
            }
        };

        let client = Client::new();
        match client
            .post("https://api.brevo.com/v3/smtp/email")
            .header("accept", "application/json")
            .header("content-type", "application/json")
            .header("api-key", &settings.mail.api_key)
            .json(&body)
            .send()
            .await
        {
            Ok(res) => {
                if !res.status().is_success() {
                    error!("Response from brevo was {}", res.status());
                    return false;
                }
                return true;
            }
            Err(e) => {
                error!("Error sending email. {}", e);
                return false;
            }
        }
    }

    /// Sends user an e-mail they can use to reset their password
    async fn password_recovery(&self, settings: &Settings, user: &User, token: &str) -> bool {
        let body_str = format!(
            r#"{{
            "sender": {{
                "name": "{}",
                "email": "{}"
            }},
            "to": [
                {{
                    "name": "{}",
                    "email": "{}"
                }}
            ],
            "subject": "Recover your Mekadomus password",
            "htmlContent": "<html><body>Hello {},<br />To reset your password, click the following link.<br /><br /><a href=\"https://console.mekadomus.com/password-recovery/{}\">Reset password</a><br><br></body></html>"
        }}"#,
            settings.mail.mailer_name,
            settings.mail.mailer_address,
            user.name,
            user.email,
            user.name,
            token
        );
        let body: serde_json::Value = match serde_json::from_str(&body_str) {
            Ok(b) => b,
            Err(e) => {
                error!("Couldn't parse json. {}", e);
                return false;
            }
        };

        let client = Client::new();
        match client
            .post("https://api.brevo.com/v3/smtp/email")
            .header("accept", "application/json")
            .header("content-type", "application/json")
            .header("api-key", &settings.mail.api_key)
            .json(&body)
            .send()
            .await
        {
            Ok(res) => {
                if !res.status().is_success() {
                    error!("Response from brevo was {}", res.status());
                    return false;
                }
                return true;
            }
            Err(e) => {
                error!("Error sending email. {}", e);
                return false;
            }
        }
    }

    async fn constant_flow_alert(
        &self,
        settings: &Settings,
        user: &User,
        meter: &FluidMeter,
    ) -> bool {
        let body_str = format!(
            r#"{{
            "sender": {{
                "name": "{}",
                "email": "{}"
            }},
            "to": [
                {{
                    "name": "{}",
                    "email": "{}"
                }}
            ],
            "subject": "Possible leak detected",
            "htmlContent": "<html><body>Hello {},<br />We've detected non-stop flow going through your <strong>{}</strong> meter.<br>Click on the link below to visualize the recent measurements:<br><br><a href=\"https://console.mekadomus.com/meter/{}\">See measurements</a></body></html>"
        }}"#,
            settings.mail.mailer_name,
            settings.mail.mailer_address,
            user.name,
            user.email,
            user.name,
            meter.name,
            meter.id
        );
        let body: serde_json::Value = match serde_json::from_str(&body_str) {
            Ok(b) => b,
            Err(e) => {
                error!("Couldn't parse json. {}", e);
                return false;
            }
        };

        let client = Client::new();
        match client
            .post("https://api.brevo.com/v3/smtp/email")
            .header("accept", "application/json")
            .header("content-type", "application/json")
            .header("api-key", &settings.mail.api_key)
            .json(&body)
            .send()
            .await
        {
            Ok(res) => {
                if !res.status().is_success() {
                    error!("Response from brevo was {}", res.status());
                    return false;
                }
                return true;
            }
            Err(e) => {
                error!("Error sending email. {}", e);
                return false;
            }
        }
    }
}
