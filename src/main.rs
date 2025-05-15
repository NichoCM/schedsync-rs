use std::sync::Arc;

use schedsync_api::build_routes;
use schedsync_api::run_server;
use schedsync_api::AppState;
use serde::{Deserialize, Serialize};

use dotenv::dotenv;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let state = Arc::new(AppState::new());
    let router = build_routes(state.clone());
    run_server(router).await;
}

#[derive(Debug, Deserialize, Serialize)]
struct NotFoundResponse {
    message: String,
}