use crate::{Config, WeatherData};
use dirs::cache_dir;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Deserialize, Serialize)]
struct CacheData {
    timestamp: chrono::DateTime<chrono::Local>,
    data: WeatherData,
}

pub fn save(data: WeatherData) {
    let file = {
        let mut path = cache_dir().unwrap();

        path.push("weather-cli.toml");

        path
    };
    let cache_data = CacheData {
        timestamp: chrono::Local::now(),
        data,
    };
    let serialized = toml::to_string(&cache_data).unwrap();

    fs::write(file, serialized).unwrap();
}

pub fn load(config: &Config) -> Option<WeatherData> {
    let file = {
        let mut path = cache_dir().unwrap();

        path.push("weather-cli.toml");

        path
    };
    if !file.exists() {
        return None;
    }
    let content = fs::read_to_string(&file).ok()?;
    let data = toml::from_str::<CacheData>(&content).ok()?;
    let now = chrono::Local::now();

    if now.signed_duration_since(data.timestamp) < config.caching_duration {
        Some(data.data)
    } else {
        None
    }
}
