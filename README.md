<div align="center">
  <a href="https://www.buymeacoffee.com/maicmi" target="_blank">
    <img src="https://cdn.buymeacoffee.com/buttons/v2/default-yellow.png" alt="Buy Me a Coffee" height="45">
  </a>
  <br/>
  <small>If you find this project helpful, consider supporting its development!</small>
</div>

<br/>

# Automatex Notifier 🚀

**Automatex Notifier** is a powerful, multi-service notification bot written in Rust. It runs continuously in the background, monitoring various external APIs for important events and sending real-time, formatted notifications to a Telegram channel.

This project was originally written in Go and has been completely rewritten in Rust using the Axum framework and Tokio for high performance, reliability, and memory safety.

![Go](https://img.shields.io/badge/Go-00ADD8?style=for-the-badge&logo=go&logoColor=white)
![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)
![Axum](https://img.shields.io/badge/Axum-7C3AED?style=for-the-badge)
![Tokio](https://img.shields.io/badge/Tokio-221B38?style=for-the-badge)

---

## ✨ Features

- **Multi-Service Monitoring**: Concurrently checks multiple data sources.
- **Real-time Notifications**: Delivers alerts to Telegram as soon as events are detected.
- **Persistent State**: Remembers previously seen events to avoid duplicate notifications, even after a restart.
- **Highly Configurable**: Easily manage API keys, chat IDs, and service settings through a `.env` file.
- **Robust and Performant**: Built with Rust for high efficiency and safety in long-running operations.
- **Extensible Architecture**: Designed with traits to make adding new notification services straightforward.

---

## 🔔 Monitored Services

Currently, Automatex Notifier supports the following services:

- **🌍 Earthquake Watcher**: Notifies about significant earthquakes (Magnitude 4.5+) worldwide using data from the USGS.
- **🚀 Rocket Launch Alerts**: Sends alerts for upcoming rocket launches within the next 24 hours, sourced from The Space Devs API.
- **☀️ Space Weather Monitor**: Reports on strong solar flares (M-Class and X-Class) that could impact Earth, using data from NASA DONKI.
- **🚨 Critical Vulnerability Scanner**: Scans for newly published critical security vulnerabilities (CVEs) from the National Vulnerability Database (NVD).

---

## 🛠️ Getting Started

Follow these instructions to get a copy of the project up and running on your local machine for development and testing purposes.

### Prerequisites

- [Rust Toolchain](https://www.rust-lang.org/tools/install) (latest stable version recommended)
- A Telegram Bot Token and a Chat ID.

### Installation & Setup

1.  **Clone the repository:**

    ```bash
    git clone https://github.com/your-username/automatex-notifier-axum.git
    cd automatex-notifier-axum
    ```

2.  **Create the environment file:**
    Copy the example environment file to create your own configuration.

    ```bash
    cp .env.example .env
    ```

    _(Note: If `.env.example` does not exist, create a new file named `.env`)_

3.  **Configure your environment:**
    Open the `.env` file and fill in your credentials and settings. At a minimum, you must provide your Telegram API key and Chat ID.

    ```env
    # .env

    # --- Telegram Bot Credentials ---
    TELEGRAM_API_KEY="YOUR_TELEGRAM_BOT_API_KEY"
    TELEGRAM_CHAT_ID="YOUR_TELEGRAM_CHAT_ID"

    # --- Common Settings ---
    BUYMEACOFFEE_URL="https://www.buymeacoffee.com/maicmi"

    # --- NASA API Key (for Space Weather) ---
    NASA_API_KEY="YOUR_NASA_API_KEY" # Get one from api.nasa.gov

    # --- Disclaimers (Optional) ---
    EARTHQUAKE_DISCLAIMER="*Disclaimer*: Data from USGS. For informational purposes only."
    # ... and so on for other services
    ```

### Running the Application

You can run the application in two modes:

- **For Development (Debug Mode):**
  This will compile and run the application quickly, with debugging information included.

  ```bash
  cargo run
  ```

- **For Production (Release Mode):**
  This will compile with full optimizations for the best performance. The first build will be slow, but subsequent runs will be fast.
  ```bash
  cargo run --release
  ```

Once running, the application will start a small web server on port `1323` and begin its monitoring cycles. You can check its status by visiting `http://localhost:1323` in your browser.

---

## 🏗️ How to Add a New Service

The project is designed to be easily extensible. To add a new notification service:

1.  Create a new file in the `src/services/` directory (e.g., `src/services/mynewservice.rs`).
2.  Define a `struct` for your service and a `struct` for its notification data.
3.  Implement the `Notification` and `NotificationService` traits for your new structs. This will involve writing the logic to fetch data from an API and format the notification message.
4.  Register your new service module in `src/services/mod.rs`.
5.  Add the configuration for your new service in `src/config.rs`.
6.  Instantiate and register your service in `src/main.rs`.

---

## 🤝 Contributing

Contributions, issues, and feature requests are welcome! Feel free to check the [issues page](https://github.com/your-username/automatex-notifier-axum/issues).

## 📜 License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details.
