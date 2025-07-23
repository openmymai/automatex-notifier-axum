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
            state: Manager::new("seen_space_weather.json", Duration::from_secs(7 * 24 * 3600)),
            config,
            client,
        }
    }
}

#[derive(Debug)]
pub struct SpaceWeatherNotification {
    id: String,
    event_type: String,
    time: i64,
    class_type: String,
    url: String,
}

impl Notification for SpaceWeatherNotification {
    fn get_unique_id(&self) -> &str { &self.id }
    fn get_timestamp(&self) -> i64 { self.time }
    fn format_message(&self) -> String {
        let title = "☀️ *Space Weather Alert* ☀️";
        let time_str = chrono::DateTime::from_timestamp(self.time, 0)
            .map(|t| t.format("%c").to_string())
            .unwrap_or_else(|| "N/A".to_string());
        
        format!(
            "{}\n\n*Event:* {}\n*Class:* {}\n*Time:* {}\n*Potential Impact:* Strong HF radio blackouts on Earth's sunlit side, increased aurora chances\\.\n*Details:* [NASA DONKI]({})",
            title,
            telegram::escape_markdown(&self.event_type),
            telegram::escape_markdown(&self.class_type),
            telegram::escape_markdown(&time_str),
            self.url
        )
    }
}

#[async_trait]
impl NotificationService for Service {
    fn name(&self) -> &str { "Space Weather" }
    fn get_config(&self) -> &ServiceConfig { &self.config }
    fn get_state_manager(&self) -> &Manager { &self.state }

    async fn check_for_notifications(&self) -> anyhow::Result<Vec<Box<dyn Notification>>> {
        #[derive(Deserialize)]
        struct FlareEvent {
            #[serde(rename = "flrID")]
            flr_id: String,
            #[serde(rename = "beginTime")]
            begin_time: String,
            #[serde(rename = "classType")]
            class_type: String,
            link: String,
        }

        let api_key = std::env::var("NASA_API_KEY")
            .unwrap_or_else(|_| "DEMO_KEY".to_string());
        let start_date = (chrono::Utc::now() - chrono::Duration::hours(24)).format("%Y-%m-%d");
        let url = format!("https://api.nasa.gov/DONKI/FLR?startDate={}&api_key={}", start_date, api_key);
        
        let res = self.client.get(&url).send().await?;
        
        if !res.status().is_success() {
             return Err(anyhow::anyhow!("NASA API returned status {}", res.status()));
        }

        let data: Vec<FlareEvent> = res.json().await?;

        let mut notifications: Vec<Box<dyn Notification>> = Vec::new();
        for event in data {
            if (event.class_type.starts_with('X') || event.class_type.starts_with('M')) && !self.state.is_seen(&event.flr_id).await {
                let event_time = match chrono::DateTime::parse_from_rfc3339(&event.begin_time) {
                    Ok(t) => t,
                    Err(e) => {
                        warn!(event_id = %event.flr_id, "Could not parse event time: {}", e);
                        continue;
                    }
                };
                
                let event_time_secs = event_time.timestamp();
                let notif = Box::new(SpaceWeatherNotification {
                    id: event.flr_id.clone(),
                    event_type: "Solar Flare Detected".to_string(),
                    time: event_time_secs,
                    class_type: event.class_type,
                    url: event.link,
                });
                notifications.push(notif);
                self.state.add(event.flr_id, event_time_secs).await;
            }
        }
        Ok(notifications)
    }
}