use std::{collections::HashMap, str::FromStr, sync::Arc};

use axum::{extract::{Path, Query}, response::Redirect};
use reqwest::StatusCode;
use serde_json::Value;
use crate::{connectors::{oauth2::Oauth2Service, ServiceType}, models::{group::Group, integration::Integration, oauth2_state::Oauth2State, oauth_integration::OauthIntegration}, AppState};
use crate::config::{Config, Oauth2Config};

/**
 * Redirect the user to the OAuth2 service.
 */
pub async fn redirect(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    axum::extract::Path(service): axum::extract::Path<String>,
    axum::extract::Query(query): axum::extract::Query<Value>,
) -> (StatusCode, Redirect) {

    let service = match Oauth2Service::from_str(&service) {
        Ok(service) => service,
        Err(_) => return (StatusCode::NOT_FOUND, Redirect::to("/404")),
    };

    println!("{:?}", query.get("group_id").unwrap_or(&Value::Null).as_i64());

    let Some(group_id) = query.get("group_id").unwrap_or(&Value::Null).as_str() else {
        return (StatusCode::BAD_REQUEST, Redirect::to("/404"));
    };

    let Ok(group_id) = group_id.parse::<i32>() else {
        return (StatusCode::BAD_REQUEST, Redirect::to("/404"));
    };

    // Find the group by the group_id
    let Some(group) = Group::find_by_id(group_id, &mut state.get_connection()) else {
        return (StatusCode::NOT_FOUND, Redirect::to("/404"));
    };

    println!("HERE 3");

    // TODO - make sure the user is authorized to access the group

    // Create a service configuration and redirect the user
    let service_config = get_service_config(service, &state.config);
    (
        StatusCode::TEMPORARY_REDIRECT,
        build_redirect(service_config, Oauth2State::new(&group, &mut state.get_connection()))
    )
}

/**
 * Handle the OAuth2 service callback.
 */
pub async fn callback(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    axum::extract::Path(service): axum::extract::Path<String>,
    axum::extract::Query(query): axum::extract::Query<Value>,
) -> (StatusCode, String) {

    // Ensure service is present
    let service = match Oauth2Service::from_str(&service) {
        Ok(service) => service,
        Err(_) => return (StatusCode::NOT_FOUND, "Service not found".to_string()),
    };

    // Ensure state is present
    let Some(state_str) = query.get("state").unwrap_or(&Value::Null).as_str() else {
        return (StatusCode::BAD_REQUEST, "Missing state".to_string());
    };

    // Query the state by the state string
    let Some(oauth2_state) = Oauth2State::find_by_string(
        &state_str.to_string(),
        &mut state.get_connection()
    ) else {
        return (StatusCode::NOT_FOUND, "State not found".to_string());
    };

    let code = query.get("code").unwrap_or(&Value::Null).as_str();
    let service_config = get_service_config(service.clone(), &state.config);
    let group = oauth2_state.get_group(&mut state.get_connection());

    // Ensure code and scope are present
    match code {
        Some(code) => {

            // Exchange code for access token and refresh token (Integration model)
            let callback = Oauth2Callback::new(code.to_string(), service.clone());
            let integration = callback.exchange_code(service_config, service, group, &state).await;

            // Ensure integration model created successfully
            match integration {
                Ok(integration) => {
                    (StatusCode::OK, "Integration created".to_string())
                },
                Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create integration".to_string())
            }
        },
        _ => (StatusCode::BAD_REQUEST, "Missing code or scope".to_string())
    }
}

/**
 * Get the service configuration from the config and service name.
 */
fn get_service_config(service: Oauth2Service, config: &Config) -> &Oauth2Config {
    match service {
        Oauth2Service::Google => &config.oauth2.google,
        Oauth2Service::Outlook => &config.oauth2.outlook,
    }
}

/**
 * Build a redirect URL for an OAuth2 service.
 */
fn build_redirect(config: &Oauth2Config, oauth2_state: Oauth2State) -> Redirect {

    let params = vec![
        ("client_id", config.client_id.to_string()),
        ("prompt", String::from("consent")),
        ("redirect_uri", config.redirect_uri.to_string()),
        ("response_type", String::from("code")),
        ("access_type", String::from("offline")),
        ("scope", config.scope.to_string()),
        ("state", oauth2_state.state.to_string()),
    ];
    let query = serde_urlencoded::to_string(params).unwrap();
    Redirect::to(format!("{}?{}", config.authorization_url, query).as_str())
}

/**
 * Struct to handle the OAuth2 callback. Once the Oauth2 callback
 * data is configured, it can exchange the token for an access token
 * and a refresh token.
 */
struct Oauth2Callback {
    code: String,
    service: Oauth2Service,
}

impl Oauth2Callback {
    fn new(code: String, service: Oauth2Service) -> Self {
        Self {
            code,
            service,
        }
    }

    /**
     * Exchange the code for an access token and refresh token. Returns
     * and Integration model which can be used elsewhere in the application.
     */
    async fn exchange_code(&self, config: &Oauth2Config, service: Oauth2Service, group: Group, state: &Arc<AppState>) -> Result<OauthIntegration, ()> {
        let params = self.get_params(config);
            
        let response = reqwest::Client::new()
            .post(config.token_url.as_str())
            .form(&params)
            .send().await
            .unwrap();

        let integration = Integration::new(&group, &mut state.get_connection(), ServiceType::from_oauth2(service.clone()));

        match response.status() {
            StatusCode::OK => {
                match response.json::<Oauth2CodeExchangeResponse>().await {
                    Ok(data) => {
                        Ok(OauthIntegration::new(
                            &integration,
                            &mut state.get_connection(),
                            service,
                            data.access_token,
                            data.refresh_token,
                            chrono::Utc::now().naive_utc() + chrono::Duration::seconds(data.expires_in as i64),
                        ))
                    },
                    Err(err) => {
                        println!("{:?}", err);
                        Err(())
                    }
                }
            },
            _ => {
                println!("{:?}", response.text().await.unwrap());
                Err(())
            }
        }
    }

    /**
     * Get the parameters for the OAuth2 token exchange.
     */
    fn get_params(&self, config: &Oauth2Config) -> HashMap<&str, String> {
        match self.service {
            Oauth2Service::Google => self.build_google_params(config),
            Oauth2Service::Outlook => self.build_outlook_params(config),
        }
    }

    /**
     * Build the parameters for the Google OAuth2 token exchange.
     */
    fn build_google_params(&self, config: &Oauth2Config) -> HashMap<&str, String> {
        HashMap::from([
            ("client_id", config.client_id.to_string()),
            ("client_secret", config.client_secret.to_string()),
            ("code", self.code.to_string()),
            ("redirect_uri", config.redirect_uri.to_string()),
            ("grant_type", String::from("authorization_code")),
            ("access_type", String::from("offline")),
        ])
    }

    /**
     * Build the parameters for the Outlook OAuth2 token exchange.
     */
    fn build_outlook_params(&self, config: &Oauth2Config) -> HashMap<&str, String> {
        HashMap::from([
            ("client_id", config.client_id.to_string()),
            ("client_secret", config.client_secret.to_string()),
            ("code", self.code.to_string()),
            ("redirect_uri", config.redirect_uri.to_string()),
            ("grant_type", String::from("authorization_code")),
        ])
    }
}

/**
 * The response from an OAuth2 code exchange.
 */
#[derive(Debug, serde::Deserialize)]
struct Oauth2CodeExchangeResponse {
    access_token: String,
    refresh_token: String,
    expires_in: u64,
    scope: String,
}