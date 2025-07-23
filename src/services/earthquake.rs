use super::{Notification, NotificationService};
use crate::{config::ServiceConfig, state::Manager, telegram};
use async_trait::async_trait;
use serde::Deserialize;
use std::time::Duration;


pub struct Service {
    state: Manager,
    config: ServiceConfig,
    client: reqwest::Client,
}

impl Service {
    pub fn new(config: ServiceConfig, client: reqwest::Client) -> Self {
        Self {
            state: Manager::new("seen_quakes.json", Duration::from_secs(72 * 3600)),
            config,
            client,
        }
    }
}

#[derive(Debug)]
pub struct EarthquakeNotification {
    id: String,
    magnitude: f64,
    location: String,
    time: i64,
    url: String,
    latitude: f64,
    longitude: f64,
}

impl Notification for EarthquakeNotification {
    fn get_unique_id(&self) -> &str { &self.id }
    fn get_timestamp(&self) -> i64 { self.time }
    fn format_message(&self) -> String {
        let title = "🌍 *Earthquake Report* 🌍";
        let time_str = chrono::DateTime::from_timestamp(self.time, 0)
            .map(|t| t.format("%c").to_string())
            .unwrap_or_else(|| "N/A".to_string());
        
        let gmaps_url = format!(
            "https://www.google.com/maps/place/{},{}/@{:.4},{:.4},5z",
            self.latitude, self.longitude, self.latitude, self.longitude
        );

        format!(
            "{}\n\n*Magnitude:* {}\n*Location:* {}\n*Time:* {}\n*Map:* [Google Maps]({}) \\| [Details on USGS]({})",
            title,
            telegram::escape_markdown(&format!("{:.2}", self.magnitude)),
            telegram::escape_markdown(&self.location),
            telegram::escape_markdown(&time_str),
            gmaps_url,
            self.url
        )
    }
}

#[async_trait]
impl NotificationService for Service {
    fn name(&self) -> &str { "Earthquake" }
    fn get_config(&self) -> &ServiceConfig { &self.config }
    fn get_state_manager(&self) -> &Manager { &self.state }

    async fn check_for_notifications(&self) -> anyhow::Result<Vec<Box<dyn Notification>>> {
        #[derive(Deserialize)]
        struct UsgsFeature {
            id: String,
            properties: UsgsProperties,
            geometry: UsgsGeometry,
        }
        #[derive(Deserialize)]
        struct UsgsProperties {
            mag: f64,
            place: String,
            time: i64,
            url: String,
        }
        #[derive(Deserialize)]
        struct UsgsGeometry {
            coordinates: [f64; 3], 
        }
        #[derive(Deserialize)]
        struct UsgsResponse {
            features: Vec<UsgsFeature>,
        }

        let res: UsgsResponse = self.client.get("https://earthquake.usgs.gov/earthquakes/feed/v1.0/summary/4.5_day.geojson").send().await?.json().await?;

        let mut notifications: Vec<Box<dyn Notification>> = Vec::new();
        for feature in res.features {
            if !self.state.is_seen(&feature.id).await {
                let event_time_secs = feature.properties.time / 1000;
                let notif = Box::new(EarthquakeNotification {
                    id: feature.id.clone(),
                    magnitude: feature.properties.mag,
                    location: feature.properties.place,
                    time: event_time_secs,
                    url: feature.properties.url,
                    latitude: feature.geometry.coordinates[1],
                    longitude: feature.geometry.coordinates[0],
                });
                notifications.push(notif);
                self.state.add(feature.id, event_time_secs).await;
            }
        }
        Ok(notifications)
    }
}