mod custom_error;
mod file_handler;
mod jobs;
mod messenger;
mod telemetry;

use std::sync::mpsc;

// pretty_env_logger related
extern crate pretty_env_logger;
#[macro_use]
extern crate log;

fn main() {
    pretty_env_logger::init();
    info!("Starting fota service");

    // Create chanel for passing notification from messenger to jobs
    let (tx_notification, rx_notification) = mpsc::channel();

    // Initialize messenger, it already handle mqtt connection on other thread
    let messenger = messenger::Messenger::new(tx_notification, "backend", "broker.emqx.io", 1883);

    // Initialize jobs and run
    let mut jobs = jobs::JobScheduler::new(messenger, rx_notification);
    jobs.run();
}
