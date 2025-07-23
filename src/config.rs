use std::env;
use std::time::Duration;
use tracing::warn;

fn get_env(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| {
        warn!("Environment variable '{}' not found, using default value.", key);
        default.to_string()
    })
}

fn get_env_duration(key: &str, default_secs: u64) -> Duration {
    env::var(key)
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .map(Duration::from_secs)
        .unwrap_or_else(|| {
            warn!("Environment variable '{}' not found or invalid, using default value.", key);
            Duration::from_secs(default_secs)
        })
}

#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub enabled: bool,
    pub check_interval: Duration,
    pub telegram_api_key: String,
    pub telegram_chat_id: String,
    pub buymeacoffee_url: String,
    pub disclaimer: String,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub earthquake: ServiceConfig,
    pub rocket_launch: ServiceConfig,
    pub space_weather: ServiceConfig,
    pub vulnerability: ServiceConfig,
}

impl Config {
    
    pub fn load() -> Self {
        let telegram_api_key = get_env("TELEGRAM_API_KEY", "");
        let telegram_chat_id = get_env("TELEGRAM_CHAT_ID", "");
        let buymeacoffee_url = get_env("BUYMEACOFFEE_URL", "");

        if telegram_api_key.is_empty() || telegram_chat_id.is_empty() {
            panic!(
                "TELEGRAM_API_KEY or TELEGRAM_CHAT_ID is missing. \
                Ensure they are set in the .env file in the project root. \
                Current working directory: {}",
                env::current_dir().unwrap().display()
            );
        }

        Config {
            earthquake: ServiceConfig {
                enabled: true, 
                check_interval: get_env_duration("EARTHQUAKE_INTERVAL_SECS", 5 * 60),
                telegram_api_key: telegram_api_key.clone(),
                telegram_chat_id: telegram_chat_id.clone(),
                buymeacoffee_url: buymeacoffee_url.clone(),
                disclaimer: get_env("EARTHQUAKE_DISCLAIMER", ""),
            },
            rocket_launch: ServiceConfig {
                enabled: true,
                check_interval: get_env_duration("ROCKETLAUNCH_INTERVAL_SECS", 15 * 60),
                telegram_api_key: telegram_api_key.clone(),
                telegram_chat_id: telegram_chat_id.clone(),
                buymeacoffee_url: buymeacoffee_url.clone(),
                disclaimer: get_env("ROCKETLAUNCH_DISCLAIMER", ""),
            },
            space_weather: ServiceConfig {
                enabled: true,
                check_interval: get_env_duration("SPACEWEATHER_INTERVAL_SECS", 30 * 60),
                telegram_api_key: telegram_api_key.clone(),
                telegram_chat_id: telegram_chat_id.clone(),
                buymeacoffee_url: buymeacoffee_url.clone(),
                disclaimer: get_env("SPACEWEATHER_DISCLAIMER", ""),
            },
            vulnerability: ServiceConfig {
                enabled: true,
                check_interval: get_env_duration("VULNERABILITY_INTERVAL_SECS", 60 * 60),
                telegram_api_key: telegram_api_key.clone(),
                telegram_chat_id: telegram_chat_id.clone(),
                buymeacoffee_url: buymeacoffee_url.clone(),
                disclaimer: get_env("VULNERABILITY_DISCLAIMER", ""),
            },
        }
    }
}