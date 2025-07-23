use crate::config::ServiceConfig;
use crate::state::Manager;
use async_trait::async_trait;
use std::fmt::Debug;

pub mod earthquake;
pub mod rocketlaunch;
pub mod spaceweather;
pub mod vulnerability;

pub trait Notification: Debug + Send + Sync {
    fn get_unique_id(&self) -> &str;
    fn get_timestamp(&self) -> i64;
    fn format_message(&self) -> String;
}

#[async_trait]
pub trait NotificationService: Send + Sync {
    fn name(&self) -> &str;
    fn get_config(&self) -> &ServiceConfig;
    fn get_state_manager(&self) -> &Manager;

    async fn check_for_notifications(&self) -> anyhow::Result<Vec<Box<dyn Notification>>>;

    async fn load_state(&self) -> anyhow::Result<()> {
        self.get_state_manager().load().await
    }

    async fn save_state(&self) -> anyhow::Result<()> {
        self.get_state_manager().save().await
    }
}