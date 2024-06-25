use config::{Config, ConfigError, File};
use serde::Deserialize;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Settings {
    pub job_max_running: u8,
    pub job_processed_interval_ms: u32,
    pub chunk_size_per_transmission: u32,
    pub http_host: String,
    pub http_port: u32,
    pub mqtt_client_id: String,
    pub mqtt_host: String,
    pub mqtt_port: u32,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(File::with_name("rocky"))
            .build()
            .unwrap();

        s.try_deserialize()
    }
}
