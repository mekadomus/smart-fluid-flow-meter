use crate::{
    api::{fluid_meter::FluidMeterAlerts, user::User},
    settings::settings::Settings,
};

use async_trait::async_trait;
use mockall::automock;
use reqwest::Client;
use serde_json::Value;
use std::fmt::Write;
use tracing::error;

pub const BYTES_PER_METER_ALERT: &'static u16 = &300;

#[automock]
#[async_trait]
pub trait MailHelper: Send + Sync {
    async fn alerts(
        &self,
        settings: &Settings,
        user: &User,
        alerts: &Vec<FluidMeterAlerts>,
    ) -> bool;
    async fn password_recovery(&self, settings: &Settings, user: &User, token: &str) -> bool;
    async fn sign_up_verification(&self, settings: &Settings, user: &User, token: &str) -> bool;
}

pub struct DefaultMailHelper;

fn alerts_mail_body(
    mailer_name: &str,
    mailer_address: &str,
    user: &User,
    alerts: &Vec<FluidMeterAlerts>,
) -> Result<Value, serde_json::Error> {
    let mut alerts_text = String::with_capacity(*BYTES_PER_METER_ALERT as usize * alerts.len());
    for a in alerts {
        let meter_alerts = a
            .alerts
            .iter()
            .map(|a| a.alert_type.to_string())
            .collect::<Vec<String>>()
            .join(", ");
        write!(
            &mut alerts_text,
            "<a href=\\\"https://console.mekadomus.com/meter/{}\\\">{}</a>: {}<br />",
            a.meter.id, a.meter.name, meter_alerts
        )
        .unwrap();
    }
    let html_content = format!("<html><body>Hello {},<br /><br />There are some alerts related to your meters:<br /><br />{}</body></html>", user.name, alerts_text);
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
        "subject": "Alerts related to your meters",
        "htmlContent": "{}"
    }}"#,
        mailer_name, mailer_address, user.name, user.email, html_content
    );
    serde_json::from_str(&body_str)
}

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

    async fn alerts(
        &self,
        settings: &Settings,
        user: &User,
        alerts: &Vec<FluidMeterAlerts>,
    ) -> bool {
        let body = match alerts_mail_body(
            &settings.mail.mailer_name,
            &settings.mail.mailer_address,
            &user,
            &alerts,
        ) {
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

#[cfg(test)]
mod tests {
    use super::alerts_mail_body;
    use crate::api::{
        alert::{
            Alert,
            AlertType::{ConstantFlow, NotReporting},
        },
        fluid_meter::{FluidMeter, FluidMeterAlerts, FluidMeterStatus::Active},
        user::{User, UserAuthProvider::Password},
    };
    use chrono::Utc;

    #[test]
    fn alerts_mail_body_success() {
        let u = User {
            id: "a".to_string(),
            provider: Password,
            name: "My name".to_string(),
            email: "a@b.com".to_string(),
            password: Some("asdf".to_string()),
            email_verified_at: Some(Utc::now().naive_utc()),
            recorded_at: Utc::now().naive_utc(),
        };
        let a = vec![
            FluidMeterAlerts {
                meter: FluidMeter {
                    id: "b".to_string(),
                    owner_id: "a".to_string(),
                    name: "meter".to_string(),
                    status: Active,
                    recorded_at: Utc::now().naive_utc(),
                    updated_at: Utc::now().naive_utc(),
                },
                alerts: vec![
                    Alert {
                        alert_type: ConstantFlow,
                    },
                    Alert {
                        alert_type: NotReporting,
                    },
                ],
            },
            FluidMeterAlerts {
                meter: FluidMeter {
                    id: "c".to_string(),
                    owner_id: "a".to_string(),
                    name: "meter_2".to_string(),
                    status: Active,
                    recorded_at: Utc::now().naive_utc(),
                    updated_at: Utc::now().naive_utc(),
                },
                alerts: vec![Alert {
                    alert_type: ConstantFlow,
                }],
            },
        ];

        assert!(alerts_mail_body("mailer", "m@i.ler", &u, &a).is_ok());
    }
}
