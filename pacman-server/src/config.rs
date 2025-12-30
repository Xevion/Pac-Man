use figment::{providers::Env, value::UncasedStr, Figment};
use serde::{Deserialize, Deserializer};
use std::env;

/// Database configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
}

/// Discord OAuth configuration
#[derive(Debug, Clone)]
pub struct DiscordConfig {
    pub client_id: String,
    pub client_secret: String,
}

/// GitHub OAuth configuration
#[derive(Debug, Clone)]
pub struct GithubConfig {
    pub client_id: String,
    pub client_secret: String,
}

/// S3 storage configuration
#[derive(Debug, Clone)]
pub struct S3Config {
    pub access_key: String,
    pub secret_access_key: String,
    pub bucket_name: String,
    pub public_base_url: String,
}

/// Main application configuration
#[derive(Debug, Clone, Deserialize)]
#[serde(from = "RawConfig")]
pub struct Config {
    /// Database configuration - if None, uses SQLite in-memory
    pub database: Option<DatabaseConfig>,
    /// Discord OAuth - if None, Discord auth is disabled
    pub discord: Option<DiscordConfig>,
    /// GitHub OAuth - if None, GitHub auth is disabled
    pub github: Option<GithubConfig>,
    /// S3 storage - if None, image storage is disabled
    pub s3: Option<S3Config>,
    /// Server port
    pub port: u16,
    /// Server host address
    pub host: std::net::IpAddr,
    /// Graceful shutdown timeout in seconds
    pub shutdown_timeout_seconds: u32,
    /// Public base URL for OAuth redirects
    pub public_base_url: String,
    /// JWT secret for session tokens
    pub jwt_secret: String,
}

/// Raw configuration loaded directly from environment variables
/// This is an intermediate representation that gets validated and converted to Config
#[derive(Debug, Deserialize)]
struct RawConfig {
    // Database
    database_url: Option<String>,

    // Discord OAuth
    #[serde(default, deserialize_with = "deserialize_optional_string_from_any")]
    discord_client_id: Option<String>,
    discord_client_secret: Option<String>,

    // GitHub OAuth
    #[serde(default, deserialize_with = "deserialize_optional_string_from_any")]
    github_client_id: Option<String>,
    github_client_secret: Option<String>,

    // S3
    s3_access_key: Option<String>,
    s3_secret_access_key: Option<String>,
    s3_bucket_name: Option<String>,
    s3_public_base_url: Option<String>,

    // Server
    #[serde(default = "default_port")]
    port: u16,
    #[serde(default = "default_host")]
    host: std::net::IpAddr,
    #[serde(default = "default_shutdown_timeout")]
    shutdown_timeout_seconds: u32,

    // Required
    public_base_url: String,
    jwt_secret: String,
}

impl From<RawConfig> for Config {
    fn from(raw: RawConfig) -> Self {
        // Validate database config
        let database = raw.database_url.map(|url| DatabaseConfig { url });

        // Validate Discord config - if any field is set, all must be set
        let discord = validate_feature_group(
            "Discord",
            &[
                ("DISCORD_CLIENT_ID", raw.discord_client_id.as_ref()),
                ("DISCORD_CLIENT_SECRET", raw.discord_client_secret.as_ref()),
            ],
        )
        .map(|_| DiscordConfig {
            client_id: raw.discord_client_id.unwrap(),
            client_secret: raw.discord_client_secret.unwrap(),
        });

        // Validate GitHub config - if any field is set, all must be set
        let github = validate_feature_group(
            "GitHub",
            &[
                ("GITHUB_CLIENT_ID", raw.github_client_id.as_ref()),
                ("GITHUB_CLIENT_SECRET", raw.github_client_secret.as_ref()),
            ],
        )
        .map(|_| GithubConfig {
            client_id: raw.github_client_id.unwrap(),
            client_secret: raw.github_client_secret.unwrap(),
        });

        // Validate S3 config - if any field is set, all must be set
        let s3 = validate_feature_group(
            "S3",
            &[
                ("S3_ACCESS_KEY", raw.s3_access_key.as_ref()),
                ("S3_SECRET_ACCESS_KEY", raw.s3_secret_access_key.as_ref()),
                ("S3_BUCKET_NAME", raw.s3_bucket_name.as_ref()),
                ("S3_PUBLIC_BASE_URL", raw.s3_public_base_url.as_ref()),
            ],
        )
        .map(|_| S3Config {
            access_key: raw.s3_access_key.unwrap(),
            secret_access_key: raw.s3_secret_access_key.unwrap(),
            bucket_name: raw.s3_bucket_name.unwrap(),
            public_base_url: raw.s3_public_base_url.unwrap(),
        });

        Config {
            database,
            discord,
            github,
            s3,
            port: raw.port,
            host: raw.host,
            shutdown_timeout_seconds: raw.shutdown_timeout_seconds,
            public_base_url: raw.public_base_url,
            jwt_secret: raw.jwt_secret,
        }
    }
}

/// Validates a feature group - returns Some(()) if all fields are set, None if all are unset,
/// or panics if only some fields are set (partial configuration).
fn validate_feature_group(feature_name: &str, fields: &[(&str, Option<&String>)]) -> Option<()> {
    let set_fields: Vec<&str> = fields.iter().filter(|(_, v)| v.is_some()).map(|(name, _)| *name).collect();

    let unset_fields: Vec<&str> = fields.iter().filter(|(_, v)| v.is_none()).map(|(name, _)| *name).collect();

    if set_fields.is_empty() {
        // All unset - feature disabled
        None
    } else if unset_fields.is_empty() {
        // All set - feature enabled
        Some(())
    } else {
        // Partial configuration - this is an error
        panic!(
            "{} configuration is incomplete. Set fields: [{}]. Missing fields: [{}]. \
             Either set all {} environment variables or none of them.",
            feature_name,
            set_fields.join(", "),
            unset_fields.join(", "),
            feature_name
        );
    }
}

// Standard User-Agent: name/version (+site)
pub const USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
    " (+https://pacman.xevion.dev)"
);

fn default_host() -> std::net::IpAddr {
    "0.0.0.0".parse().unwrap()
}

fn default_port() -> u16 {
    3000
}

fn default_shutdown_timeout() -> u32 {
    5
}

fn deserialize_optional_string_from_any<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde_json::Value;

    let value: Option<Value> = Option::deserialize(deserializer)?;
    match value {
        Some(Value::String(s)) => Ok(Some(s)),
        Some(Value::Number(n)) => Ok(Some(n.to_string())),
        Some(Value::Null) | None => Ok(None),
        _ => Err(serde::de::Error::custom("Expected string, number, or null")),
    }
}

pub fn load_config() -> Config {
    Figment::new()
        .merge(Env::raw().map(|key| {
            if key == UncasedStr::new("RAILWAY_DEPLOYMENT_DRAINING_SECONDS") {
                "SHUTDOWN_TIMEOUT_SECONDS".into()
            } else {
                key.into()
            }
        }))
        .extract()
        .expect("Failed to load config")
}

/// Create a minimal config for testing with specific overrides
/// This is useful for tests that don't need full configuration
#[cfg(test)]
pub fn test_config() -> Config {
    Config {
        database: None,
        discord: None,
        github: None,
        s3: None,
        port: 0,
        host: "127.0.0.1".parse().unwrap(),
        shutdown_timeout_seconds: 5,
        public_base_url: "http://localhost:3000".to_string(),
        jwt_secret: "test_jwt_secret_key_for_testing_only".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_feature_group_all_set() {
        let a = Some("value_a".to_string());
        let b = Some("value_b".to_string());
        let result = validate_feature_group("Test", &[("A", a.as_ref()), ("B", b.as_ref())]);
        assert!(result.is_some());
    }

    #[test]
    fn test_validate_feature_group_none_set() {
        let result = validate_feature_group("Test", &[("A", None), ("B", None)]);
        assert!(result.is_none());
    }

    #[test]
    #[should_panic(expected = "Test configuration is incomplete")]
    fn test_validate_feature_group_partial_panics() {
        let a = Some("value_a".to_string());
        validate_feature_group("Test", &[("A", a.as_ref()), ("B", None)]);
    }

    #[test]
    fn test_minimal_config() {
        let config = test_config();
        assert!(config.database.is_none());
        assert!(config.discord.is_none());
        assert!(config.github.is_none());
        assert!(config.s3.is_none());
    }
}
