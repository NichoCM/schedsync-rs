use std::collections::HashMap;

use reqwest::StatusCode;
use serde::Deserialize;

use crate::{config::Oauth2Config, models::{calendar::CalendarResult, oauth_integration::OauthIntegration}};

use super::{Oauth2ConnectorError, Oauth2ServiceConnector};

pub struct GoogleConnector {
    pub client: reqwest::Client,
    pub config: Oauth2Config
}

impl Oauth2ServiceConnector for GoogleConnector {
    fn get_config(&self) -> &Oauth2Config {
        &self.config
    }

    async fn revoke_access_token(
        &self,
        integration: &OauthIntegration
    ) -> Result<(), Oauth2ConnectorError> {
        
        let config = self.get_config();

        let query = HashMap::from([
            ("token", integration.access_token.as_str()),
        ]);

        let params = std::collections::HashMap::from([
            ("client_id", config.client_id.as_str()),
            ("client_secret", config.client_secret.as_str()),
        ]);

        // Revoke the access token
        let response = reqwest::Client::new()
            .post(config.revoke_url.as_str())
            .query(&query)
            .form(&params)
            .send().await;
        
        let response = match response {
            Ok(response) => response,
            Err(err) => {
                return Err(Oauth2ConnectorError::NetworkError(err));
            }
        };

        if response.status() != StatusCode::OK {
            return Err(Oauth2ConnectorError::InvalidStatusError(
                response.status(),
                response.text().await.unwrap_or("".to_string()),
            ));
        }

        Ok(())
    }

    async fn get_calendars(&self, integration: &OauthIntegration) -> Result<Vec<CalendarResult>, Oauth2ConnectorError>{
        
        let mut last_page_token: Option<String> = None;
        let mut items: Vec<GoogleCalendarResult> = Vec::new();

        // Loop through the calendar list and get all calendars by page
        loop {
            let result = match self.get_calendar_page(last_page_token, integration).await {
                Ok(result) => result,
                Err(err) => {
                    return Err(err);
                }
            };
            last_page_token = result.nextPageToken;
            items.extend(result.items);
            if last_page_token.is_none() { break; }
        }

        // Map the calendars to the CalendarResult struct
        Ok(items.into_iter().map(|e| CalendarResult {
            external_id: e.id.clone(),
            name: e.summary.clone(),
            background_color: e.backgroundColor.clone(),
            foreground_color: e.foregroundColor.clone(),
        }).collect::<Vec<CalendarResult>>())
       
    }

}

impl GoogleConnector {

    pub fn new(config: Oauth2Config) -> Self {
        Self {
            config,
            client: reqwest::Client::new()
        }
    }

    /**
     * Google restricts the amount of items that can be retrieved in a single request. This
     * function will paginate through the calendar list and return all calendars.
     */
    async fn get_calendar_page(
        &self, sync_token: Option<String>,
        integration: &OauthIntegration,
    ) -> Result<CalendarListResponse, Oauth2ConnectorError> {
        
        // Get the list of calendars from the API
        let response = self.client
            .get("https://www.googleapis.com/calendar/v3/users/me/calendarList");

        let response = match sync_token {
            Some(token) => {
                response.query(&HashMap::from([
                    ("pageToken", token),
                ]))
            },
            None => response
        }
            .header("Authorization", format!("Bearer {}", integration.access_token))
            .send().await;

        // Guard to get the response
        let response = match response {
            Ok(response) => response,
            Err(err) => {
                return Err(Oauth2ConnectorError::NetworkError(err));
            }
        };
    
        // Ensure the status code is OK
        if response.status() != StatusCode::OK { 
            return Err(Oauth2ConnectorError::InvalidStatusError(
                response.status(),
                response.text().await.unwrap_or("".to_string()),
            ))
        }

        // Guard to get the JSON response
        match response.json::<CalendarListResponse>().await {
            Ok(result) => Ok(result),
            Err(err) => {
                return Err(Oauth2ConnectorError::ParseResultError(err));
            }
        }
    }
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)] // Allow camel case for Google API response
struct CalendarListResponse {
    items: Vec<GoogleCalendarResult>,
    etag: String,
    nextPageToken: Option<String>,
    nextSyncToken: Option<String>,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)] // Allow camel case for Google API response
struct GoogleCalendarResult {
    id: String,
    summary: String,
    backgroundColor: String,
    foregroundColor: String,
    colorId: String,
    accessRole: String,
    primary: bool,
    selected: bool,
    timeZone: String,
}

#[derive(Deserialize)]
#[allow(non_snake_case)] // Allow camel case for Google API response
struct GoogleNotificationSettings {
    notifications: Vec<GoogleNotification>
}

#[derive(Deserialize)]
#[allow(non_snake_case)] // Allow camel case for Google API response
struct GoogleNotification {
    method: String,
    r#type: String
}

#[derive(Deserialize)]
#[allow(non_snake_case)] // Allow camel case for Google API response
struct GoogleRefreshTokenResponse {
    access_token: String,
    expires_in: u64,
    scope: String,
    refresh_token: String,
}

/*
 * Sample calendar list response
Array [
  Object {
    "accessRole": String("owner"),
    "backgroundColor": String("#9a9cff"),
    "colorId": String("17"),
    "conferenceProperties": Object {
      "allowedConferenceSolutionTypes": Array [
        String("hangoutsMeet")
      ]},
      "defaultReminders": Array [
        Object {"method": String("popup"), "minutes": Number(30)}
      ],
      "etag": String("\"1401131222998000\""),
      "foregroundColor": String("#000000"),
      "id": String("<redacted>@gmail.com"),
      "kind": String("calendar#calendarListEntry"),
      "notificationSettings": Object {
        "notifications": Array [
          Object {"method": String("email"), "type": String("eventCreation")},
          Object {"method": String("email"), "type": String("eventChange")},
          Object {"method": String("email"), "type": String("eventCancellation")},
          Object {"method": String("email"), "type": String("eventResponse")}
        ]
      },
      "primary": Bool(true),
      "selected": Bool(true),
      "summary": String("<redacted>@gmail.com"),
      "timeZone": String("America/Toronto")
    }
]
*/