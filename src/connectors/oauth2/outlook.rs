use crate::{config::Oauth2Config, models::{calendar::CalendarResult, integration::Integration, oauth_integration::OauthIntegration}};

use super::{Oauth2ConnectorError, Oauth2ServiceConnector};

pub struct OutlookConnector {

    pub config: Oauth2Config

}

impl Oauth2ServiceConnector for OutlookConnector {
    fn get_config(&self) -> &Oauth2Config {
        &self.config
    }

    async fn revoke_access_token(&self, integration: &OauthIntegration) -> Result<(), Oauth2ConnectorError> {
        // Do nothing, as Outlook does not support token revocation
        Err(Oauth2ConnectorError::TokenRevocationError(
            "Outlook does not support token revocation".to_string()
        ))
    }

    async fn get_calendars(&self, integration: &OauthIntegration) -> Result<Vec<CalendarResult>, Oauth2ConnectorError> {
        Ok(Vec::new())
    }

}

impl OutlookConnector {
    pub fn new(config: Oauth2Config) -> Self {
        Self {
            config
        }
    }
}
