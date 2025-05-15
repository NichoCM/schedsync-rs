use std::sync::Arc;

use axum::{middleware as axum_middleware, Router};
use config::Config;
use diesel::{r2d2::{ConnectionManager, Pool}, Connection};

pub mod models;
pub mod schema;
pub mod controllers;
pub mod config;
pub mod connectors;
pub mod helper;
pub mod db;
pub mod middleware;

// Test imports
pub mod test_util;

pub struct AppState {
    pub config: Config,
    pub connection_pool: Pool<ConnectionManager<db::Connection>>,
}

impl AppState {
    pub fn get_connection(&self) -> db::PooledConnection {
        self.connection_pool.get().expect("Could not get connection from pool")
    }

    pub fn new() -> Self {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        crate::db::Connection::establish(&database_url)
            .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));
    
        AppState {
            config: Config::new(),
            connection_pool: db::get_connection_pool(database_url),
        }
    }
}

/**
 * Build the routes for the application.
 */
pub fn build_routes(state: Arc<AppState>) -> Router<()> {
    let update_routes = Router::new()
    .route("/calendar/:integration", axum::routing::get({
        
    }))
    .route("/event", axum::routing::get({
        
    }));

    let api_routes = Router::new()
        .route("/group", axum::routing::post(controllers::group::store))
            .layer(axum_middleware::from_fn_with_state(state.clone(), middleware::authorize_middleware));


    let oauth_routes = Router::new()
        .route("/:service", axum::routing::get(controllers::oauth2::redirect))
        .route("/:service/callback", axum::routing::get(controllers::oauth2::callback));

    // Compose the routes
    let group = Router::new()
        .nest("/update", update_routes)
        .nest("/oauth2", oauth_routes)
        .nest("/api", api_routes);

    Router::new()
        .nest("/", group)
        .with_state(state)
}

/**
 * Run the server with the given router.
 */
pub async fn run_server(router: Router) {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, router).await.unwrap();
}