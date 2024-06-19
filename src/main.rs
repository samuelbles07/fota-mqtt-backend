mod custom_error;
mod file_handler;
mod jobs;
mod messenger;
mod telemetry;

use std::sync::mpsc;

fn main() {
    let (tx_notification, rx_notification) = mpsc::channel();

    let messenger = messenger::Messenger::new(tx_notification, "backend", "broker.emqx.io", 1883);

    let mut jobs = jobs::JobScheduler::new(messenger, rx_notification);
    jobs.run();
}
