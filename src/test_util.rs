use crate::models::{app::App, app_key::AppKey};
use base64::prelude::*;

pub fn generate_basic_header(app: &App, key: &AppKey) -> String {
    let token = format!("{}:{}", app.client_id, key.key);
    format!("Basic {}", BASE64_STANDARD.encode(token))
}