use serde::Deserialize;
use crate::verification_handler::verification_config;
use crate::data_handler::data_config;




#[derive(Clone, Debug, Deserialize)]
pub struct WebhookConfig {
    verification: verification_config::VerificationConfig,
    pub(crate) data: data_config::DataMap,
}

impl WebhookConfig {
  
    pub fn set_token(&mut self, token: String) {
        self.verification.set_expected_token(token)
    }

    pub fn get_verification_config(&self) -> &verification_config::VerificationConfig {
        &self.verification
    }

    pub fn get_verification_config_owned(&self) -> verification_config::VerificationConfig {
        self.verification.clone()
    }
}

