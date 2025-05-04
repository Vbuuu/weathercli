use crate::providers::{OpenMeteo, WeatherProvider};
use chrono::Duration;
use dirs::home_dir;
use reqwest::blocking;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::fs;

mod caching;
mod providers;

mod duration_format {
    use crate::parse_duration;
    use chrono::Duration;
    use serde::{Deserializer, Serializer, de};
    use std::fmt;
    use std::fmt::Formatter;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let hours = duration.num_hours();

        if hours > 0 && duration.num_minutes() % 60 == 0 {
            serializer.serialize_str(&format!("{}h", hours))
        } else {
            let minutes = duration.num_minutes();

            serializer.serialize_str(&format!("{}min", minutes))
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct DurationVisitor;

        impl de::Visitor<'_> for DurationVisitor {
            type Value = Duration;

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str("a duration formated as '1h' or '30min'")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                parse_duration(value).ok_or_else(|| E::custom("failed to parse duration"))
            }
        }

        deserializer.deserialize_str(DurationVisitor)
    }
}

#[derive(Deserialize, Serialize)]
enum ConfigWeatherProvider {
    #[serde(rename = "open-meteo")]
    OpenMeteo,
    #[serde(rename = "open-weather-map")]
    OpenWeatherMap,
}

#[derive(Deserialize, Serialize)]
#[serde(untagged)]
#[derive(Clone)]
enum ConfigLocation {
    City(String, String),  // City, Country
    Coordinates(f32, f32), // Latitude, Longitude
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum ConfigUnits {
    Metric,
    Imperial,
}

#[derive(Deserialize, Serialize)]
enum ConfigTimeFormat {
    #[serde(rename = "24h")]
    _24H,
    #[serde(rename = "12h")]
    _12H,
}

#[derive(Deserialize, Serialize)]
struct Config {
    provider: ConfigWeatherProvider,
    api_key: Option<String>,
    location: Option<ConfigLocation>,
    units: ConfigUnits,
    time_format: ConfigTimeFormat,
    #[serde(with = "duration_format")]
    caching_duration: Duration,
}

#[derive(Deserialize, Serialize)]
struct WeatherData {
    temperature: String,
    feels_like: String,
    wind_speed: String,
    wind_direction: String,
    condition: WeatherCondition,
}

#[derive(Deserialize, Serialize)]
enum WeatherCondition {
    Clear,
    PartlyCloudy,
    Overcast,
    Foggy,
    Drizzle,
    Rainy,
    Snowy,
    SnowGrains,
    RainShowers,
    SnowShowers,
    Thunderstorms,
    Unknown,
}

#[derive(Deserialize)]
struct MullvadResponse {
    latitude: f32,
    longitude: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_key: None,
            provider: ConfigWeatherProvider::OpenMeteo,
            location: None,
            units: ConfigUnits::Metric,
            time_format: ConfigTimeFormat::_24H,
            caching_duration: Duration::hours(1),
        }
    }
}

impl Display for ConfigWeatherProvider {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "https://{}",
            match self {
                ConfigWeatherProvider::OpenMeteo => "open-meteo.com".to_string(),
                ConfigWeatherProvider::OpenWeatherMap => "openweathermap.org".to_string(),
            }
        )
    }
}

impl Display for WeatherCondition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                WeatherCondition::Clear => "Clear",
                WeatherCondition::PartlyCloudy => "Partly Cloudy",
                WeatherCondition::Overcast => "Overcast",
                WeatherCondition::Foggy => "Foggy",
                WeatherCondition::Drizzle => "Drizzle",
                WeatherCondition::Rainy => "Rainy",
                WeatherCondition::Snowy => "Snowy",
                WeatherCondition::SnowGrains => "Snow Grains",
                WeatherCondition::RainShowers => "Showers",
                WeatherCondition::SnowShowers => "Showers",
                WeatherCondition::Thunderstorms => "Thunderstorm",
                WeatherCondition::Unknown => "Unknown",
            }
        )
    }
}

impl Config {
    fn resolve_location(&mut self) {
        if self.location.is_none() {
            let res: MullvadResponse = blocking::get("https://ipv6.am.i.mullvad.net/json") // Seems to give the best results
                .unwrap()
                .json()
                .unwrap();

            self.location = Some(ConfigLocation::Coordinates(res.latitude, res.longitude));
        }
    }
}

impl ConfigUnits {
    fn temperature(&self) -> String {
        match self {
            ConfigUnits::Metric => "celsius",
            ConfigUnits::Imperial => "fahrenheit",
        }
        .to_string()
    }

    fn speed(&self) -> String {
        match self {
            ConfigUnits::Metric => "kmh",
            ConfigUnits::Imperial => "mph",
        }
        .to_string()
    }

    fn to_string(&self) -> String {
        match self {
            ConfigUnits::Metric => "metric",
            ConfigUnits::Imperial => "imperial",
        }
        .to_string()
    }
}

fn main() {
    let mut config = read_config();
    let provider: Box<dyn WeatherProvider> = match config.provider {
        ConfigWeatherProvider::OpenMeteo => Box::new(OpenMeteo),
        ConfigWeatherProvider::OpenWeatherMap => Box::new(providers::OpenWeatherMap),
    };

    let mut cache_hit = false;

    let weather = if let Some(data) = caching::load(&config) {
        data
    } else {
        config.resolve_location();
        cache_hit = true;
        provider.fetch_weather(&config).unwrap()
    };

    let current_time = match config.time_format {
        ConfigTimeFormat::_24H => {
            let now = chrono::Local::now();
            now.format("%H:%M").to_string()
        }
        ConfigTimeFormat::_12H => {
            let now = chrono::Local::now();
            now.format("%I:%M %p").to_string()
        }
    };

    println!(
        "{:<14}feels like {}",
        weather.temperature, weather.feels_like
    );
    println!(
        "{:<14}wind speed {} ({})",
        weather.condition.to_string(),
        weather.wind_speed,
        weather.wind_direction
    );
    println!("{:<14}{}", current_time, config.provider);

    if cache_hit {
        caching::save(weather);
    }
}

fn read_config() -> Config {
    let file = {
        let mut path = home_dir().unwrap();

        path.push(".config");
        path.push("weather-cli.toml");

        path
    };

    if !file.exists() {
        println!("Config file does not exist.");
    }

    let content = fs::read_to_string(&file).unwrap();

    toml::from_str::<Config>(&content).unwrap_or_else(|err| {
        println!("Failed to parse config file. {}", err);
        Config::default()
    })
}

fn parse_duration(string: &str) -> Option<Duration> {
    if let Some(h_pos) = string.find("h") {
        let hours: i64 = string[..h_pos].parse().ok()?;
        Some(Duration::hours(hours))
    } else if let Some(min_pos) = string.find("min") {
        let minutes: i64 = string[..min_pos].parse().ok()?;
        Some(Duration::minutes(minutes))
    } else {
        None
    }
}
