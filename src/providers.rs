use crate::{Config, ConfigLocation, ConfigUnits, WeatherData};
use reqwest::{blocking, Error as ReqwestError};
use serde::{Deserialize, Serialize};

pub trait WeatherProvider {
    fn fetch_weather(&self, config: &Config) -> Result<WeatherData, ReqwestError>;
}

pub struct OpenMeteo;
pub struct OpenWeatherMap;

impl WeatherProvider for OpenMeteo {
    fn fetch_weather(&self, config: &Config) -> Result<WeatherData, ReqwestError> {
        let (latitude, longitude) = match &config.location.clone().unwrap() {
            ConfigLocation::Coordinates(lat, lon) => (*lat, *lon),
            ConfigLocation::City(city, country) => {
                let url = format!(
                    "https://geocoding-api.open-meteo.com/v1/search?name={}&count=1&format=json&countryCode={}",
                    city, country
                );

                #[derive(Serialize, Deserialize)]
                struct Struct {
                    pub latitude: f32,
                    pub longitude: f32,
                }

                #[derive(Serialize, Deserialize)]
                struct Root {
                    pub results: Vec<Struct>,
                }

                let res: Root = blocking::get(url)?.json()?;

                let data = res
                    .results
                    .first()
                    .expect("No City found, check your config");

                (data.latitude, data.longitude)
            }
        };

        let url = format!(
            "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&models=best_match&current=apparent_temperature,wind_speed_10m,wind_direction_10m,temperature_2m,weather_code&temperature_unit={}&wind_speed_unit={}",
            latitude,
            longitude,
            &config.units.temperature(),
            &config.units.speed(),
        );

        #[derive(Serialize, Deserialize)]
        struct Current {
            pub time: String,
            pub interval: i32,
            pub apparent_temperature: f32,
            pub wind_speed_10m: f32,
            pub wind_direction_10m: i16,
            pub temperature_2m: f32,
            pub weather_code: i32,
        }

        #[derive(Serialize, Deserialize)]
        struct CurrentUnits {
            pub time: String,
            pub interval: String,
            pub apparent_temperature: String,
            pub wind_speed_10m: String,
            pub wind_direction_10m: String,
            pub temperature_2m: String,
            pub weather_code: String,
        }

        #[derive(Serialize, Deserialize)]
        struct Root {
            pub current_units: CurrentUnits,
            pub current: Current,
        }

        let res: Root = blocking::get(url)?.json()?;

        Ok(WeatherData {
            temperature: format!(
                "{}{}",
                res.current.temperature_2m as i32, res.current_units.temperature_2m
            ),
            feels_like: format!(
                "{}{}",
                res.current.apparent_temperature as i32, res.current_units.apparent_temperature
            ),
            wind_speed: format!(
                "{}{}",
                res.current.wind_speed_10m, res.current_units.wind_speed_10m
            ),
            wind_direction: degree_to_direction(res.current.wind_direction_10m),
            condition: {
                use crate::WeatherCondition::*;
                match res.current.weather_code {
                    0 | 1 => Clear,
                    2 => PartlyCloudy,
                    3 => Overcast,
                    45 | 48 => Foggy,
                    51 | 53 | 55 | 56 | 57 => Drizzle,
                    61 | 63 | 65 | 66 | 67 => Rainy,
                    71 | 73 | 75 => Snowy,
                    77 => SnowGrains,
                    80..=82 => RainShowers,
                    85 | 86 => SnowShowers,
                    95 | 96 | 99 => Thunderstorms,
                    _ => Unknown,
                }
            },
        })
    }
}

impl WeatherProvider for OpenWeatherMap {
    fn fetch_weather(&self, config: &Config) -> Result<WeatherData, ReqwestError> {
        let api_key = if let Some(api_key) = &config.api_key {
            api_key
        } else {
            panic!("Missing API key");
        };

        let location = match &config.location.clone().unwrap() {
            ConfigLocation::Coordinates(lat, lon) => {
                format!("lat={}&lon={}", lat, lon)
            }
            ConfigLocation::City(city, country) => {
                format!("q={},{}", city, country)
            }
        };

        let url = format!(
            "https://api.openweathermap.org/data/2.5/weather?{}&appid={}&units={}",
            location,
            api_key,
            &config.units.to_string()
        );

        #[derive(Serialize, Deserialize)]
        struct Wind {
            pub deg: i16,
            pub speed: f32,
        }

        #[derive(Serialize, Deserialize)]
        struct Struct {
            pub description: String,
            pub icon: String,
            pub id: i64,
            pub main: String,
        }

        #[derive(Serialize, Deserialize)]
        struct Main {
            pub feels_like: f64,
            pub temp: f64,
        }

        #[derive(Serialize, Deserialize)]
        struct Root {
            pub main: Main,
            pub weather: Vec<Struct>,
            pub wind: Wind,
        }

        let res: Root = blocking::get(url)?.json()?;

        let temp_unit = match &config.units {
            ConfigUnits::Imperial => "°F",
            ConfigUnits::Metric => "°C",
        };

        let wind_speed = match &config.units {
            ConfigUnits::Metric => format!("{:.1}km/h", res.wind.speed), // Documentation says that it returns the ms, but it seems like it returns km/h
            ConfigUnits::Imperial => format!("{}mph", res.wind.speed),
        };

        Ok(WeatherData {
            temperature: format!("{}{}", res.main.temp as i32, temp_unit),
            feels_like: format!("{}{}", res.main.feels_like as i32, temp_unit),
            wind_speed,
            wind_direction: degree_to_direction(res.wind.deg),
            condition: {
                use crate::WeatherCondition::*;
                match res.weather.first() {
                    Some(weather) => match weather.id {
                        200..=232 => Thunderstorms,
                        300..=321 => Drizzle,
                        500..=504 | 511 => Rainy,
                        520..=531 => RainShowers,
                        600..=602 | 611..=616 => Snowy,
                        620..=622 => SnowShowers,
                        741 => Foggy,
                        800 => Clear,
                        801..=802 => PartlyCloudy,
                        803..=804 => Overcast,
                        _ => Unknown,
                    },
                    None => Unknown,
                }
            },
        })
    }
}

fn degree_to_direction(degree: i16) -> String {
    match degree {
        0..=22 => "N",
        23..=67 => "NE",
        68..=112 => "E",
        113..=157 => "SE",
        158..=202 => "S",
        203..=247 => "SW",
        248..=292 => "W",
        293..=337 => "NW",
        _ => "N",
    }
    .to_string()
}
