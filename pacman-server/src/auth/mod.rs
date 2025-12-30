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
    /// Create a new AuthRegistry with providers based on configuration.
    /// Only providers with complete configuration will be registered.
    pub fn new(config: &Config) -> Result<Self, oauth2::url::ParseError> {
        let http = reqwest::ClientBuilder::new()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("HTTP client should build");

        let mut providers: HashMap<&'static str, Arc<dyn provider::OAuthProvider>> = HashMap::new();

        // Register GitHub provider if configured
        if let Some(github_config) = &config.github {
            let github_client: BasicClient<EndpointSet, EndpointNotSet, EndpointNotSet, EndpointNotSet, EndpointSet> =
                BasicClient::new(oauth2::ClientId::new(github_config.client_id.clone()))
                    .set_client_secret(oauth2::ClientSecret::new(github_config.client_secret.clone()))
                    .set_auth_uri(oauth2::AuthUrl::new("https://github.com/login/oauth/authorize".to_string())?)
                    .set_token_uri(oauth2::TokenUrl::new(
                        "https://github.com/login/oauth/access_token".to_string(),
                    )?)
                    .set_redirect_uri(
                        oauth2::RedirectUrl::new(format!("{}/auth/github/callback", config.public_base_url))
                            .expect("Invalid redirect URI"),
                    );

            providers.insert("github", github::GitHubProvider::new(github_client, http.clone()));
            tracing::info!("GitHub OAuth provider registered");
        }

        // Register Discord provider if configured
        if let Some(discord_config) = &config.discord {
            let discord_client: BasicClient<EndpointSet, EndpointNotSet, EndpointNotSet, EndpointNotSet, EndpointSet> =
                BasicClient::new(oauth2::ClientId::new(discord_config.client_id.clone()))
                    .set_client_secret(oauth2::ClientSecret::new(discord_config.client_secret.clone()))
                    .set_auth_uri(oauth2::AuthUrl::new("https://discord.com/api/oauth2/authorize".to_string())?)
                    .set_token_uri(oauth2::TokenUrl::new("https://discord.com/api/oauth2/token".to_string())?)
                    .set_redirect_uri(
                        oauth2::RedirectUrl::new(format!("{}/auth/discord/callback", config.public_base_url))
                            .expect("Invalid redirect URI"),
                    );

            providers.insert("discord", discord::DiscordProvider::new(discord_client, http));
            tracing::info!("Discord OAuth provider registered");
        }

        if providers.is_empty() {
            tracing::warn!("No OAuth providers configured - authentication will be unavailable");
        }

        Ok(Self { providers })
    }

    pub fn get(&self, id: &str) -> Option<&Arc<dyn provider::OAuthProvider>> {
        self.providers.get(id)
    }

    pub fn values(&self) -> impl Iterator<Item = &Arc<dyn provider::OAuthProvider>> {
        self.providers.values()
    }

    /// Get the number of registered providers
    pub fn len(&self) -> usize {
        self.providers.len()
    }
}
