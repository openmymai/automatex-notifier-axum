mod config;
mod services;
mod state;
mod telegram;

use axum::{routing::get, Router};
use services::{
    earthquake, rocketlaunch, spaceweather, vulnerability, Notification, NotificationService,
};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::time;
use tracing::{error, info, instrument};

#[tokio::main]
async fn main() {
    match dotenvy::dotenv() {
        Ok(path) => info!("Loaded .env file from {}", path.display()),
        Err(e) => error!("Could not load .env file: {}", e),
    }

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cfg = Arc::new(config::Config::load());

    let client = reqwest::Client::new();

    let mut available_services: Vec<Box<dyn NotificationService>> = Vec::new();
    if cfg.earthquake.enabled {
        available_services.push(Box::new(earthquake::Service::new(cfg.earthquake.clone(), client.clone())));
    }
    if cfg.rocket_launch.enabled {
        available_services.push(Box::new(rocketlaunch::Service::new(cfg.rocket_launch.clone(), client.clone())));
    }
    if cfg.space_weather.enabled {
        available_services.push(Box::new(spaceweather::Service::new(cfg.space_weather.clone(), client.clone())));
    }
    if cfg.vulnerability.enabled {
        available_services.push(Box::new(vulnerability::Service::new(cfg.vulnerability.clone(), client.clone())));
    }
    
    for s in available_services {
        tokio::spawn(start_service_monitor(Arc::from(s)));
    }

    let app = Router::new().route("/", get(handler));
    let addr = SocketAddr::from(([0, 0, 0, 0], 8010));
    info!("Starting Automatex Notifier web server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn handler() -> &'static str {
    "Automatex Notifier is running!"
}

#[instrument(skip(s), fields(service = %s.name()))]
async fn start_service_monitor(s: Arc<Box<dyn NotificationService>>) {
    info!("Initializing service");
    if let Err(e) = s.load_state().await {
        error!("Error loading state: {:?}", e);
    }

    let check_interval = s.get_config().check_interval;
    let mut interval = time::interval(check_interval);
    interval.set_missed_tick_behavior(time::MissedTickBehavior::Delay);

    run_check(s.clone()).await;

    loop {
        interval.tick().await;
        run_check(s.clone()).await;
    }
}

#[instrument(skip(s), fields(service = %s.name()))] 
async fn run_check(s: Arc<Box<dyn NotificationService>>) {
    info!("Checking for new notifications...");
    let notifications = match s.check_for_notifications().await {
        Ok(n) => n,
        Err(e) => {
            error!("Error checking for notifications: {:?}", e);
            return;
        }
    };

    if notifications.is_empty() {
        info!("No new notifications found.");
        return;
    }

    info!("Found {} new notification(s).", notifications.len());

    let cfg = s.get_config();
    let sender = telegram::Sender::new(cfg.telegram_api_key.clone(), cfg.telegram_chat_id.clone());

    for n in notifications {
        send_single_notification(&sender, n, cfg).await;
        time::sleep(Duration::from_secs(1)).await;
    }

    if let Err(e) = s.save_state().await {
        error!("Error saving state: {:?}", e);
    }
}

async fn send_single_notification(
    sender: &telegram::Sender,
    n: Box<dyn Notification>,
    cfg: &config::ServiceConfig,
) {
    let mut full_message = n.format_message();

    if !cfg.buymeacoffee_url.is_empty() || !cfg.disclaimer.is_empty() {
        full_message.push_str("\n\n");
        full_message.push_str(&telegram::escape_markdown("--------------------"));
    }
    if !cfg.buymeacoffee_url.is_empty() {
        full_message.push_str("\n\n*Like this service?*");
        full_message.push_str(&format!(
            "\n[Buy Me a Coffee ☕]({})",
            cfg.buymeacoffee_url
        ));
    }
    if !cfg.disclaimer.is_empty() {
        full_message.push_str("\n\n");
        full_message.push_str(&cfg.disclaimer);
    }

    if let Err(e) = sender.send(&full_message).await {
        error!(notification_id = %n.get_unique_id(), "Failed to send notification: {:?}", e);
    } else {
        info!(notification_id = %n.get_unique_id(), "Successfully sent notification.");
    }
}