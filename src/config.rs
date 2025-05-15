#[derive(Debug, Clone)]

pub struct Config {
    pub oauth2: Oauth2ConfigGroup,
}

impl Config {
    pub fn new() -> Self {
        Self {
            oauth2: Oauth2ConfigGroup::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Oauth2ConfigGroup {
    pub google: Oauth2Config,
    pub outlook: Oauth2Config,
}

impl Oauth2ConfigGroup {
    fn new() -> Self {
        Self {
            google: Oauth2Config::google(),
            outlook: Oauth2Config::outlook(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Oauth2Config {
    pub authorization_url: String,
    pub token_url: String,
    pub revoke_url: String,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub scope: String,
}

impl Oauth2Config {

    fn outlook() -> Self {
        Self::build(
            "https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize".to_string(),
            "https://login.microsoftonline.com/consumers/oauth2/v2.0/token".to_string(),
            "https://login.microsoftonline.com/consumers/oauth2/v2.0/revoke".to_string(),
            std::env::var("OUTLOOK_CLIENT_ID").expect("OUTLOOK_API_TOKEN must be set."),
            std::env::var("OUTLOOK_CLIENT_SECRET").expect("OUTLOOK_API_SECRET must be set."),
            std::env::var("OUTLOOK_REDIRECT_URI").expect("OUTLOOK_REDIRECT_URI must be set."),
            std::env::var("OUTLOOK_SCOPES").expect("OUTLOOK_SCOPES must be set."),
        )
    }

    fn google() -> Self {
        Self::build(
            "https://accounts.google.com/o/oauth2/auth".to_string(),
            "https://oauth2.googleapis.com/token".to_string(),
            "https://accounts.google.com/o/oauth2/revoke".to_string(),
            std::env::var("GOOGLE_CLIENT_ID").expect("GOOGLE_API_TOKEN must be set."),
            std::env::var("GOOGLE_CLIENT_SECRET").expect("GOOGLE_API_SECRET must be set."),
            std::env::var("GOOGLE_REDIRECT_URI").expect("GOOGLE_REDIRECT_URI must be set."),
            std::env::var("GOOGLE_SCOPES").expect("GOOGLE_SCOPES must be set."),
        )
    }

    fn build(authorization_url: String, token_url: String, revoke_url: String, client_id: String, client_secret: String, redirect_uri: String, scope: String) -> Self {
        Self {
            authorization_url,
            token_url,
            revoke_url,
            client_id,
            client_secret,
            redirect_uri,
            scope,
        }

    }
}