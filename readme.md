## Konfiguration

Weather-cli verwendet eine TOML-Konfigurationsdatei, die unter `~/.config/weather-cli.toml` gespeichert wird.

### Konfigurationsoptionen

```toml
# Wetterdatenanbieter: "open-meteo" oder "open-weather-map"
provider = "open-meteo"

# API-Schlüssel (nur für OpenWeatherMap erforderlich) (optional)
api_key = "dein_api_schlüssel"

# Standort: Entweder als Koordinaten oder Stadt-Land-Paar (optional)
# Option 1: Koordinaten
location = [48.137154, 11.576124]  # München (Breitengrad, Längengrad)

# Option 2: Stadt und Land (hier müssen manchmal zwei api anfragen gemacht werden)
# location = ["Berlin", "DE"]

# Maßeinheiten: "metric" (°C, km/h) oder "imperial" (°F, mph)
units = "metric"

# Zeitformat: "24h" oder "12h"
time_format = "24h"

# Caching-Dauer in Stunden oder Minuten (z.B. "1h" oder "15min")
caching_duration = "1h"
```

Falls keine `location` angegeben wird, werden deine aktuellen Koordinaten über den Mullvad-Dienst ermittelt.

## Ausgabe

Weather-cli zeigt folgende Informationen an:

- Aktuelle Temperatur und gefühlte Temperatur
- Wetterbedingung und Windgeschwindigkeit (mit Richtung)
- Aktuelle Zeit und verwendete Datenquelle

## Caching

Weather-cli speichert abgerufene Wetterdaten zwischen und verwendet diese bei wiederholten Aufrufen.
