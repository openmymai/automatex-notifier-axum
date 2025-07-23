use super::{Notification, NotificationService};
use crate::{config::ServiceConfig, state::Manager, telegram};
use async_trait::async_trait;
use serde::Deserialize;
use std::time::Duration;
use tracing::warn;

pub struct Service {
    state: Manager,
    config: ServiceConfig,
    client: reqwest::Client,
}

impl Service {
    pub fn new(config: ServiceConfig, client: reqwest::Client) -> Self {
        Self {
            state: Manager::new("seen_launches.json", Duration::from_secs(30 * 24 * 3600)),
            config,
            client, 
        }
    }
}

#[derive(Debug)]
pub struct RocketLaunchNotification {
    id: String,
    name: String,
    agency: String,
    vehicle: String,
    launch_time: i64,
    watch_url: Option<String>,
}

impl Notification for RocketLaunchNotification {
    fn get_unique_id(&self) -> &str { &self.id }
    fn get_timestamp(&self) -> i64 { self.launch_time }
    fn format_message(&self) -> String {
        let title = "🚀 *Rocket Launch Alert* 🚀";
        let time_str = chrono::DateTime::from_timestamp(self.launch_time, 0)
            .map(|t| t.format("%c").to_string())
            .unwrap_or_else(|| "N/A".to_string());
        
        let mut msg = format!(
            "{}\n\n*Mission:* {}\n*Agency:* {}\n*Vehicle:* {}\n*Launch Time:* {}",
            title,
            telegram::escape_markdown(&self.name),
            telegram::escape_markdown(&self.agency),
            telegram::escape_markdown(&self.vehicle),
            telegram::escape_markdown(&time_str)
        );

        if let Some(url) = &self.watch_url {
            msg.push_str(&format!("\n*Watch Live:* [Click Here]({})", url));
        }
        msg
    }
}

#[async_trait]
impl NotificationService for Service {
    fn name(&self) -> &str { "Rocket Launch" }
    fn get_config(&self) -> &ServiceConfig { &self.config }
    fn get_state_manager(&self) -> &Manager { &self.state }

    async fn check_for_notifications(&self) -> anyhow::Result<Vec<Box<dyn Notification>>> {
        #[derive(Deserialize)]
        struct LaunchResult {
            id: String,
            name: String,
            net: String, 
            launch_service_provider: LaunchServiceProvider,
            rocket: Rocket,
            #[serde(rename = "vidURLs", default)]
            vid_urls: Vec<VidURL>,
        }

        #[derive(Deserialize)]
        struct LaunchServiceProvider {
            name: String,
        }

        #[derive(Deserialize)]
        struct Rocket {
            configuration: RocketConfiguration,
        }
        
        #[derive(Deserialize)]
        struct RocketConfiguration {
            full_name: String,
        }

        #[derive(Deserialize)]
        struct VidURL {
            url: String,
        }

        #[derive(Deserialize)]
        struct LaunchResponse {
            results: Vec<LaunchResult>,
        }
        
        let now = chrono::Utc::now();
        let window_end = (now + chrono::Duration::hours(24)).to_rfc3339();
        let url = format!(
            "https://ll.thespacedevs.com/2.2.0/launch/upcoming/?limit=10&window_end={}",
            window_end
        );

        let res: LaunchResponse = self.client.get(&url).send().await?.json().await?;

        let mut notifications: Vec<Box<dyn Notification>> = Vec::new();
        for result in res.results {
            let launch_time = match chrono::DateTime::parse_from_rfc3339(&result.net) {
                Ok(t) => t,
                Err(e) => {
                    warn!(launch_id = %result.id, "Could not parse launch time: {}", e);
                    continue;
                }
            };
            
            let time_until_launch = launch_time.signed_duration_since(now);
            
            if !self.state.is_seen(&result.id).await && time_until_launch < chrono::Duration::from_std(self.config.check_interval + Duration::from_secs(60)).unwrap() {
                let launch_time_secs = launch_time.timestamp();
                let notif = Box::new(RocketLaunchNotification {
                    id: result.id.clone(),
                    name: result.name,
                    agency: result.launch_service_provider.name,
                    vehicle: result.rocket.configuration.full_name,
                    launch_time: launch_time_secs,
                    watch_url: result.vid_urls.first().map(|v| v.url.clone()),
                });
                notifications.push(notif);
                self.state.add(result.id, launch_time_secs).await;
            }
        }
        Ok(notifications)
    }
}