use std::sync::Arc;

use axum::Json;
use reqwest::StatusCode;

use crate::{middleware::AuthenticatedApp, models::group::Group, AppState};

pub async fn store(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    axum::extract::Extension(authenticated): axum::extract::Extension<AuthenticatedApp>,
) -> Result<Json<Group>, StatusCode> {
    let group = authenticated.app.create_group(&mut state.get_connection());
    Ok(Json::from(group))
}