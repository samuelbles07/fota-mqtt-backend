use config::{Config, ConfigError, File};
use serde::Deserialize;
use std::sync::OnceLock;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Settings {
    pub job_max_running: u8,
    pub job_processed_interval_ms: u64,
    pub chunk_size_per_transmission: u16,
    pub http_host: String,
    pub http_port: u32,
    pub mqtt_client_id: String,
    pub mqtt_host: String,
    pub mqtt_port: u16,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(File::with_name("rocky.toml"))
            .build()
            .unwrap();

        s.try_deserialize()
    }
}

pub fn settings() -> &'static Settings {
    static S: OnceLock<Settings> = OnceLock::new();
    S.get_or_init(|| Settings::new().unwrap())
}
