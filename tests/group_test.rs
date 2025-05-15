use std::sync::Arc;

use axum::{body::Body, http::Request, http};
use dotenv::dotenv;
use http_body_util::BodyExt;
use reqwest::{Method, StatusCode};
use schedsync_api::{build_routes, models::{app::App, group::Group}, test_util, AppState};
use tower::util::ServiceExt;

mod common;

#[tokio::test]
async fn create_group_invalid_client() {
    dotenv().ok();
    let state = Arc::new(AppState::new());
    let router = build_routes(state.clone());
    let response = router
        .oneshot(
            Request::builder()
                .uri("/api/group")
                .method(Method::POST)
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::empty())
                .unwrap()
        ).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

struct Test(i32);

#[tokio::test]
async fn create_group_valid_client() {
    dotenv().ok();
    let state = Arc::new(AppState::new());
    let router = build_routes(state.clone());
    let app = App::new(&mut state.get_connection());
    let app_key = app.create_key(&mut state.get_connection());

    let response = router
        .oneshot(
            Request::builder()
                .uri("/api/group")
                .method(Method::POST)
                .header(http::header::CONTENT_TYPE, "application/json")
                .header(http::header::AUTHORIZATION, test_util::generate_basic_header(&app, &app_key))
                .body(Body::empty())
                .unwrap()
        ).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Attempt to deserialize the response body into a Group model
    let _: Group = serde_json::from_slice(
        &response.into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
    ).unwrap();
}