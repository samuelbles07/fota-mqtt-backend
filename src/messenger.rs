use rumqttc::{Client, Connection, Event, MqttOptions, QoS};
use std::error::Error;
use std::thread;
use std::time::Duration;

use crate::telemetry::Telemetry;

pub struct Messenger {
    mqttc: Client,
}

impl Messenger {
    pub fn new(client_id: String, mqtt_host: String, mqtt_port: u16) -> Self {
        let mut mqtt_options = MqttOptions::new(client_id, mqtt_host, mqtt_port);
        mqtt_options.set_keep_alive(Duration::from_secs(5));
        // TODO: Add last will if necessary later

        let (client, connection) = Client::new(mqtt_options, 1);
        thread::spawn(move || Messenger::run_connection(connection));

        client
            .subscribe("/fota/cmd_resp/+", QoS::AtMostOnce) // TODO: Topic from other const
            .unwrap(); // TODO: Handle return result

        Self { mqttc: client }
    }

    pub fn send(&mut self, telemetry: Telemetry) -> Result<(), Box<dyn Error>> {
        let result =
            self.mqttc
                .publish(telemetry.topic, QoS::AtLeastOnce, false, telemetry.payload)?;
        Ok(result)
    }

    fn run_connection(mut connection: Connection) {
        for (i, notification) in connection.iter().enumerate() {
            match notification {
                Ok(event) => {
                    // println!("{i}. Notification = {notif:?}");
                    Messenger::handle_notification_event(event);
                }
                Err(error) => {
                    println!("{i}. Notification = {error:?}");
                    return;
                }
            }
        }
    }

    fn handle_notification_event(event: Event) {
        let Event::Incoming(incoming) = event else {
            return;
        };

        match incoming {
            rumqttc::Packet::Publish(data) => {
                let notification = Telemetry {
                    topic: data.topic,
                    payload: data.payload.to_vec(),
                };
                println!("notification {:?}", notification);
            }
            _ => return,
        }
    }
}
