mod google;
mod outlook;

use std::{str::FromStr, sync::Arc};

use chrono::{Duration, Local};
use diesel::{deserialize::FromSqlRow, expression::AsExpression, serialize::ToSql, Queryable};
use crate::{config::Oauth2Config, models::{calendar::CalendarResult, oauth_integration::OauthIntegration}, AppState};

pub trait Oauth2ServiceConnector {

    async fn new_access_token(&self, integration: &mut OauthIntegration, state: &Arc<AppState>) -> Result<OauthIntegration, Oauth2ConnectorError> {

        let config = self.get_config();
        
        let params = std::collections::HashMap::from([
            ("client_id", config.client_id.as_str()),
            ("client_secret", config.client_secret.as_str()),
            ("refresh_token", integration.refresh_token.as_str()),
            ("grant_type", "refresh_token"),
        ]);

        // Refresh the access token
        let response = reqwest::Client::new()
            .post(config.token_url.as_str())
            .form(&params)
            .send().await
            .unwrap();

        if response.status() != reqwest::StatusCode::OK {
            return Err(Oauth2ConnectorError::InvalidStatusError(
                response.status(),
                response.text().await.unwrap(),
            ));
        }

        let integration = match response.json::<Oauth2TokenResponse>().await {
            Ok(data) => {
                integration.access_token = data.access_token;
                integration.expires_at = (Local::now() + Duration::seconds(data.expires_in)).naive_utc();
                integration.save(&mut state.get_connection());
                Ok(integration.clone())
            },
            Err(err) => {
                println!("{:?}", err);
                Err(Oauth2ConnectorError::ParseResultError(err))
            }
        };
        
        integration
    }

    /**
     * Revoke the access token for the given integration.
     */
    async fn revoke_access_token(&self, integration: &OauthIntegration) -> Result<(), Oauth2ConnectorError>;

    /**
     * Get the configuration for the service.
     */
    fn get_config(&self) -> &Oauth2Config;

    async fn get_calendars(&self, integration: &OauthIntegration) -> Result<Vec<CalendarResult>, Oauth2ConnectorError>;
}

/**
 * The response from an OAuth2 token exchange.
 */
#[derive(Debug, serde::Deserialize)]
struct Oauth2TokenResponse {
    access_token: String,
    expires_in: i64,
}

/**
 * The error types for connectors
 */
 #[derive(Debug)]
pub enum Oauth2ConnectorError {
    TokenRevocationError(String),
    ParseResultError(reqwest::Error),
    NetworkError(reqwest::Error),
    InvalidStatusError(reqwest::StatusCode, String),
}

/**
 * Enum to represent the OAuth2 services.
 */
#[derive(Clone, Debug, FromSqlRow, AsExpression, PartialEq, Eq)]
#[diesel(sql_type = diesel::sql_types::SmallInt)]
pub enum Oauth2Service {
    Google,
    Outlook,
}

/**
 * Convert an i16 used in the database to an Oauth2Service.
 */
impl Queryable<diesel::sql_types::SmallInt, crate::db::Backend> for Oauth2Service {
    type Row = i16;
    fn build(row: Self::Row) -> Result<Oauth2Service, Box<(dyn std::error::Error + Send + Sync + 'static)>> {
        match row {
            1 => Ok(Self::Google),
            2 => Ok(Self::Outlook),
            _ => Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid Oauth2Service value")))
        }
    }
}

impl ToSql<diesel::sql_types::SmallInt, crate::db::Backend> for Oauth2Service {
    fn to_sql<'b>(&'b self, out: &mut diesel::serialize::Output<'b, '_, crate::db::Backend>) -> diesel::serialize::Result {
        match self {
            Oauth2Service::Google => {
                let _ = ToSql::<diesel::sql_types::SmallInt, crate::db::Backend>::to_sql(&1, out);
                Ok(diesel::serialize::IsNull::No)
            },
            Oauth2Service::Outlook => {
                let _ = ToSql::<diesel::sql_types::SmallInt, crate::db::Backend>::to_sql(&2, out);
                Ok(diesel::serialize::IsNull::No)
            },
        }
    }
}

impl ToString for Oauth2Service {
    fn to_string(&self) -> String {
        match self {
            Oauth2Service::Google => String::from("google"),
            Oauth2Service::Outlook => String::from("outlook"),
        }
    }
}

impl FromStr for Oauth2Service {
    type Err = ();
    fn from_str(service: &str) -> Result<Self, Self::Err> {
        match service {
            "google" => Ok(Self::Google),
            "outlook" => Ok(Self::Outlook),
            _ => Err(()),
        }
    }
}