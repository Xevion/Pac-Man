#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use std::collections::HashMap;
use std::sync::Arc;

use oauth2::{basic::BasicClient, EndpointNotSet, EndpointSet};

use crate::config::Config;

#[cfg_attr(coverage_nightly, coverage(off))]
pub mod discord;
#[cfg_attr(coverage_nightly, coverage(off))]
pub mod github;
pub mod provider;

type OAuthClient =
    BasicClient<oauth2::EndpointSet, oauth2::EndpointNotSet, oauth2::EndpointNotSet, oauth2::EndpointNotSet, oauth2::EndpointSet>;

pub struct AuthRegistry {
    pub providers: HashMap<&'static str, Arc<dyn provider::OAuthProvider>>,
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
        providers.insert("github", github::GitHubProvider::new(github_client, http.clone()));

        // Discord OAuth client
        let discord_client: BasicClient<EndpointSet, EndpointNotSet, EndpointNotSet, EndpointNotSet, EndpointSet> =
            BasicClient::new(oauth2::ClientId::new(config.discord_client_id.clone()))
                .set_client_secret(oauth2::ClientSecret::new(config.discord_client_secret.clone()))
                .set_auth_uri(oauth2::AuthUrl::new("https://discord.com/api/oauth2/authorize".to_string())?)
                .set_token_uri(oauth2::TokenUrl::new("https://discord.com/api/oauth2/token".to_string())?)
                .set_redirect_uri(
                    oauth2::RedirectUrl::new(format!("{}/auth/discord/callback", config.public_base_url))
                        .expect("Invalid redirect URI"),
                );
        providers.insert("discord", discord::DiscordProvider::new(discord_client, http));

        Ok(Self { providers })
    }

    pub fn get(&self, id: &str) -> Option<&Arc<dyn provider::OAuthProvider>> {
        self.providers.get(id)
    }

    pub fn values(&self) -> impl Iterator<Item = &Arc<dyn provider::OAuthProvider>> {
        self.providers.values()
    }
}
