use std::collections::HashMap;
use std::sync::Arc;

use oauth2::{basic::BasicClient, EndpointNotSet, EndpointSet};

use crate::config::Config;

pub mod github;
pub mod provider;

pub struct AuthRegistry {
    providers: HashMap<&'static str, Arc<dyn provider::OAuthProvider>>,
}

impl AuthRegistry {
    pub fn new(config: &Config) -> Result<Self, oauth2::url::ParseError> {
        let http = reqwest::ClientBuilder::new()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("HTTP client should build");

        let github_client: BasicClient<EndpointSet, EndpointNotSet, EndpointNotSet, EndpointNotSet, EndpointSet> =
            BasicClient::new(oauth2::ClientId::new(config.github_client_id.clone()))
                .set_client_secret(oauth2::ClientSecret::new(config.github_client_secret.clone()))
                .set_auth_uri(oauth2::AuthUrl::new("https://github.com/login/oauth/authorize".to_string())?)
                .set_token_uri(oauth2::TokenUrl::new(
                    "https://github.com/login/oauth/access_token".to_string(),
                )?)
                .set_redirect_uri(
                    oauth2::RedirectUrl::new(format!("{}/auth/github/callback", config.public_base_url))
                        .expect("Invalid redirect URI"),
                );

        let mut providers: HashMap<&'static str, Arc<dyn provider::OAuthProvider>> = HashMap::new();
        providers.insert("github", github::GitHubProvider::new(github_client, http));

        Ok(Self { providers })
    }

    pub fn get(&self, id: &str) -> Option<&Arc<dyn provider::OAuthProvider>> {
        self.providers.get(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&'static str, &Arc<dyn provider::OAuthProvider>)> {
        self.providers.iter().map(|(k, v)| (*k, v))
    }
}
