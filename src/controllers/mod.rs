pub mod oauth2;
pub mod group;

pub enum RequestError {
    BearerTokenMissing,
    AppNotFound,
    AuthorizationError,
}