mod custom_error;
mod file_handler;
mod httpserver;
mod jobs;
mod messenger;
mod settings;
mod telemetry;

use lazy_static::lazy_static;
use std::sync::mpsc;

// pretty_env_logger related
extern crate pretty_env_logger;
#[macro_use]
extern crate log;

// Globalize config
lazy_static! {
    static ref CONFIG: settings::Settings = settings::Settings::new().unwrap();
}

fn main() {
    pretty_env_logger::init();
    info!("Starting fota service");

    // Create channel for passing notification from messenger to jobs thread
    let (tx_notification, rx_notification) = mpsc::channel();
    // Create channel for passing new job from http server to jobs thread
    let (tx_new_job, rx_new_job) = mpsc::channel();

    // Initialize messenger, it already handle mqtt connection on other thread
    let messenger = messenger::Messenger::new(tx_notification, "backend", "broker.emqx.io", 1883);

    // Initialize jobs and run
    let jobs = jobs::JobScheduler::new(messenger, rx_notification, rx_new_job);
    jobs.run();

    let mut http = httpserver::HTTPServer::new("127.0.0.1", 7878, tx_new_job);
    http.run();
}
