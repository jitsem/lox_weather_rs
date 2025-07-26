use chrono::{TimeZone, Utc};
use reqwest::Client;
use serde::Deserialize;
use std::env;
use std::fmt::Write;

const PIRATEWEATHER_API_KEY_NAME: &str = "PIRATEWEATHER_API_KEY";
const PIRATEWEATHER_BASE_URL: &str = "https://api.pirateweather.net/forecast";
const CITY_NAME_NAME: &str = "CITY_NAME";
const COUNTRY_NAME_NAME: &str = "COUNTRY_NAME";

const CITY_NAME_FALLBACK: &str = "Toembakistan";
const COUNTRY_NAME_FALLBACK: &str = "Balacambia";
const LOX_LICENSE_EXP_DATE: &str = "2099-06-30";

#[derive(Deserialize, Debug)]
struct WeatherReport {
    latitude: f64,
    longitude: f64,
    offset: f64,
    hourly: HourlySummary,
    daily: DailySummary,
    currently: Currently,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Currently {
    temperature: f64,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct HourlyData {
    time: i64,
    icon: String,
    precip_intensity: f64,
    precip_probability: f64,
    temperature: f64,
    apparent_temperature: f64,
    humidity: f64,
    pressure: f64,
    wind_speed: f64,
    wind_gust: f64,
    wind_bearing: f64,
    cloud_cover: f64,
    uv_index: f64,
}

#[derive(Deserialize, Debug)]
struct HourlySummary {
    data: Vec<HourlyData>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct DailyData {
    sunrise_time: i64,
    sunset_time: i64,
}

#[derive(Deserialize, Debug)]
struct DailySummary {
    data: Vec<DailyData>,
}

pub struct DataCreationOptions {
    pub above_sea_level: u32,
    pub latitude: f64,
    pub longitude: f64,
}

pub async fn create_weather_xml(
    query: DataCreationOptions,
) -> Result<String, Box<dyn std::error::Error>> {
    //Get the country and city from the environment
    let city_name = match env::var(CITY_NAME_NAME) {
        Ok(val) => val,
        _ => CITY_NAME_FALLBACK.to_owned(),
    };

    let country_name = match env::var(COUNTRY_NAME_NAME) {
        Ok(val) => val,
        _ => COUNTRY_NAME_FALLBACK.to_owned(),
    };

    let weather_report = get_response_from_provider(query.latitude, query.longitude).await?;

    let mut csv = String::new();
    csv += "<mb_metadata>\n";
    csv += "id;name;longitude;latitude;height (m.asl.);country;timezone;utc-timedifference;sunrise;sunset;local date;weekday;local time;temperature(C);feeledTemperature(C);windspeed(km/h);winddirection(degr);wind gust(km/h);low clouds(%);medium clouds(%);high clouds(%);precipitation(mm);probability of Precip(%);snowFraction;sea level pressure(hPa);relative humidity(%);CAPE;picto-code;radiation (W/m2);\n";
    csv += "</mb_metadata><valid_until>";
    csv += LOX_LICENSE_EXP_DATE;
    csv += "</valid_until>\n";
    csv += "<station>\n";

    let (lon, ew) = if weather_report.longitude < 0.0 {
        (-weather_report.longitude, 'W')
    } else {
        (weather_report.longitude, 'E')
    };
    let (lat, ns) = if weather_report.latitude < 0.0 {
        (-weather_report.latitude, 'S')
    } else {
        (weather_report.latitude, 'N')
    };

    let sunrise_time = match Utc
        .timestamp_opt(weather_report.daily.data[0].sunrise_time, 0)
        .single()
    {
        Some(dt) => dt.format("%H:%M").to_string(),
        None => return Err("Invalid sunrise timestamp".into()),
    };
    let sunset_time = match Utc
        .timestamp_opt(weather_report.daily.data[0].sunset_time, 0)
        .single()
    {
        Some(dt) => dt.format("%H:%M").to_string(),
        None => return Err("Invalid sunset timestamp".into()),
    };

    csv += &format!(
        ";{};{:.2}°{};{:.2}°{};{};{};CEST;UTC{:+.1};{};{};\n",
        city_name,
        lon,
        ew,
        lat,
        ns,
        query.above_sea_level,
        country_name,
        weather_report.offset,
        sunrise_time,
        sunset_time
    );
    for hourly in &weather_report.hourly.data {
        let icon_id = get_loxone_weather_icon(hourly);
        let timestamp = match Utc.timestamp_opt(hourly.time, 0).single() {
            Some(dt) => dt.format("%d.%m.%Y;%a;%H").to_string(),
            None => return Err("Invalid hourly timestamp".into()),
        };
        writeln!(&mut csv, "{};{:5.1};{:5.1};{:3.0};{:3.0};{:3.0};{:3.0};{:3.0};{:3.0};{:5.1};{:3.0};{:3.1};{:4.0};{:3.0};{:6};{};{:4.0};",
    timestamp,
    hourly.temperature,
    hourly.apparent_temperature,
    hourly.wind_speed,
    hourly.wind_bearing,
    hourly.wind_gust,
    0.0,
    hourly.cloud_cover * 100.0,
    0.0,
    hourly.precip_intensity,
    hourly.precip_probability * 100.0,
    0.0,
    hourly.pressure,
    hourly.humidity * 100.0,
    0,
    icon_id,
    (hourly.uv_index * 100.0) as i64
    ).map_err(|_| "Could not fil in hourly data")?;
    }

    csv += "</station>\n";
    Ok(csv)
}

pub async fn get_current_temp(
    query: DataCreationOptions,
) -> Result<String, Box<dyn std::error::Error>> {
    let weather_report = get_response_from_provider(query.latitude, query.longitude).await?;
    Ok(weather_report.currently.temperature.to_string())
}

async fn get_response_from_provider(
    lat: f64,
    long: f64,
) -> Result<WeatherReport, Box<dyn std::error::Error>> {
    let api_key = match env::var(PIRATEWEATHER_API_KEY_NAME) {
        Ok(val) => val,
        Err(e) => return Err(format!("couldn't find {PIRATEWEATHER_API_KEY_NAME}: {e}").into()),
    };

    let url = format!(
        "{}/{}/{},{}{}",
        PIRATEWEATHER_BASE_URL, &api_key, lat, long, "?&units=ca"
    );

    let data = Client::new().get(url).send().await?.text().await?;

    let weather_report: Result<WeatherReport, serde_json::Error> = serde_json::from_str(&data);

    match weather_report {
        Ok(x) => Ok(x),
        Err(e) => Err(format!("Something went wrong parsing json: {}", e).into()),
    }
}

fn get_loxone_weather_icon(weather_report_hourly: &HourlyData) -> i32 {
    let icon_pirateweather = &weather_report_hourly.icon;
    let mut icon_id = 7;

    match icon_pirateweather.as_str() {
        "clear-day" | "clear-night" => icon_id = 1, // wolkenlos
        "rain" => icon_id = 23,                     // Regen
        "snow" => icon_id = 24,                     // Schneefall
        "sleet" | "hail" => icon_id = 35,           // Schneeregen
        "fog" => icon_id = 16,                      // Nebel
        "cloudy" | "partly-cloudy-day" | "partly-cloudy-night" => icon_id = 7, // Wolkig
        "thunderstorm" => icon_id = 28,             // Gewitter
        _ => (),
    }

    // Fix the cloud cover icon
    if icon_id == 7 {
        let cloud_cover = weather_report_hourly.cloud_cover;
        icon_id = if cloud_cover < 0.125 {
            1 // Wolkenlos und sonnig
        } else if cloud_cover < 0.5 {
            3 // Heiter und leicht bewölkt
        } else if cloud_cover < 0.75 {
            9 // bewölkt bis stark bewölkt
        } else if cloud_cover < 0.875 {
            19 // Stark bewölkt
        } else {
            22 // fast bedeckt und bedeckt
        }
    }

    // Add rain, if necessary
    if icon_id == 23 && weather_report_hourly.precip_intensity > 0.0 {
        icon_id = if weather_report_hourly.precip_intensity < 0.5 {
            33 // Leichter Regen
        } else if weather_report_hourly.precip_intensity <= 4.0 {
            23 // Regen
        } else {
            25 // Starker Regen
        }
    }

    icon_id
}
