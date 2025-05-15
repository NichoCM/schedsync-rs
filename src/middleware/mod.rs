use std::sync::Arc;

use axum::{extract::{Request, State, Extension}, middleware::Next, response::IntoResponse};
use reqwest::{header, StatusCode};
use base64::prelude::*;

use crate::{models::{app::App, app_key::AppKey}, AppState};

const BASIC_PREFIX: &str = "Basic ";

/**
 * Middleware to authorize the client using Basic Authorization.
 */
pub async fn authorize_middleware(State(state): State<Arc<AppState>>, mut req: Request, next: Next) -> Result<impl IntoResponse, (StatusCode, String)> {
    let Some(authorization) = req.headers().get(header::AUTHORIZATION) else {
        return Err((StatusCode::UNAUTHORIZED, "Missing Authorization Header".to_string()))
    };

    let Ok(authorization) = authorization.to_str() else {
        return Err((StatusCode::UNAUTHORIZED, "Malformed Authorization Header".to_string()))
    };

    // Basic authorization required
    if !authorization.starts_with(BASIC_PREFIX) {
        return Err((StatusCode::UNAUTHORIZED, "Only Basic Authorization is Supported".to_string()))
    };

    // Extract the encoded credential
    let Some(encoded) = authorization.get(BASIC_PREFIX.len()..) else {
        println!("Failed to extract encoded credential");
        return Err((StatusCode::UNAUTHORIZED, "Malformed Authorization Header".to_string()))
    };

    // Decode the base64 credential into a Vec<u8>
    let Ok(decoded) = BASE64_STANDARD.decode(encoded) else {
        return Err((StatusCode::UNAUTHORIZED, "Malformed Authorization Header".to_string()))
    };

    // Convert the Vec<u8> into a String
    let Ok(credential) = String::from_utf8(decoded) else {
        return Err((StatusCode::UNAUTHORIZED, "Malformed Authorization Header".to_string()))
    };

    // Split the credential into client_id and client_secret
    let parts: Vec<&str> = credential.split(':').collect();
    let (client_id, client_secret) = match parts.as_slice() {
        [client_id, client_secret] => (*client_id, *client_secret),
        _ => return Err((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))
    };

    // Find the App by the client_id
    let Some(app) = App::find_by_client_id(String::from(client_id), &mut state.get_connection()) else {
        return Err((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))
    };

    // Find the AppKey by the client_secret and App
    let Some(key) = AppKey::find_by_app_key(
        &app,
        &String::from(client_secret),
        &mut state.get_connection()
    ) else {
        return Err((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))
    };

    // Add the AuthenticatedApp to the extensions for use in the controllers
    req.extensions_mut().insert(AuthenticatedApp {
        app,
        key,
    });

    // Continue to the next middleware
    Ok(next.run(req).await)
}


#[derive(Clone)]
pub struct AuthenticatedApp {
    pub app: App,
    pub key: AppKey,
}