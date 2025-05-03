use serde::Deserialize;
use crate::verification_handler::verification_config;
use crate::data_handler::data_config;
use crate::polling_handler::polling_config::PollingConfig;

#[derive(Clone, Debug, Deserialize)]
pub struct WebhookConfig {
    verification: verification_config::VerificationConfig,
    data: data_config::DataMap,
    #[serde(skip)]
    polling: PollingConfig,
}

impl WebhookConfig {

    pub fn set_token(&mut self, token: String) {
        self.verification.set_expected_token(token)
    }

    pub fn get_verification_config(&self) -> &verification_config::VerificationConfig {
        &self.verification
    }

    pub fn get_data_config(&self) -> &data_config::DataMap {
        &self.data
    }
    
    pub fn get_verification_config_owned(&self) -> verification_config::VerificationConfig {
        self.verification.clone()
    }
    
    pub fn get_polling_config_owned(&self) -> PollingConfig {
        self.polling.clone()
    }
    
    pub fn init_polling_config(&mut self) {
        self.polling = PollingConfig::new();
    }
}

