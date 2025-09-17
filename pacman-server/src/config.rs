use figment::{providers::Env, value::UncasedStr, Figment};
use serde::{Deserialize, Deserializer};

#[derive(Debug, Deserialize)]
pub struct Config {
    // Database URL
    pub database_url: String,
    // Discord Credentials
    #[serde(deserialize_with = "deserialize_string_from_any")]
    pub discord_client_id: String,
    pub discord_client_secret: String,
    // GitHub Credentials
    #[serde(deserialize_with = "deserialize_string_from_any")]
    pub github_client_id: String,
    pub github_client_secret: String,
    // S3 Credentials
    pub s3_access_key: String,
    pub s3_secret_access_key: String,
    pub s3_endpoint: String,
    pub s3_region: String,
    // Server Details
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_host")]
    pub host: std::net::IpAddr,
    #[serde(default = "default_shutdown_timeout")]
    pub shutdown_timeout_seconds: u32,
}

fn default_host() -> std::net::IpAddr {
    "0.0.0.0".parse().unwrap()
}

fn default_port() -> u16 {
    3000
}

fn default_shutdown_timeout() -> u32 {
    5
}

fn deserialize_string_from_any<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    use serde_json::Value;

    let value = Value::deserialize(deserializer)?;
    match value {
        Value::String(s) => Ok(s),
        Value::Number(n) => Ok(n.to_string()),
        _ => Err(serde::de::Error::custom("Expected string or number")),
    }
}

pub fn load_config() -> Config {
    Figment::new()
        .merge(Env::raw().map(|key| {
            if key == UncasedStr::new("RAILWAY_DEPLOYMENT_DRAINING_SECONDS") {
                "SHUTDOWN_TIMEOUT".into()
            } else {
                key.into()
            }
        }))
        .extract()
        .expect("Failed to load config")
}
