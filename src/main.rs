use axum::http::HeaderValue;
use axum::{
    debug_handler, extract::Extension, extract::Query, response::IntoResponse, response::Response,
    routing::get, Router,
};
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

use serde::Deserialize;

use chrono::format::strftime::StrftimeItems;
use chrono::{DateTime, Local, NaiveTime, TimeZone, Utc};

use std::fmt::Write;

const CITY_NAME: &str = "Brussels";
const COUNTRY_NAME: &str = "Belgium";
const LOX_LICENSE_EXP_DATE: &str = "2024-06-30";
const TEST_DATA_CSV: &str = r#"<mb_metadata>
id;name;longitude;latitude;height (m.asl.);country;timezone;utc-timedifference;sunrise;sunset;local date;weekday;local time;temperature(C);feeledTemperature(C);windspeed(km/h);winddirection(degr);wind gust(km/h);low clouds(%);medium clouds(%);high clouds(%);precipitation(mm);probability of Precip(%);snowFraction;sea level pressure(hPa);relative humidity(%);CAPE;picto-code;radiation (W/m2);
</mb_metadata><valid_until>2024-06-30</valid_until>
<station>
;Brussels;15.00°E;48.00°N;70.1104;Belgium;CEST;UTC+2.0;06:00;20:00;
30.06.2023;Fri;12; 23.4; 25.4;  5;180;  7;  0; 45;  0;  0.2;  5;0.0;1012; 35;     0; 1;   0;
30.06.2023;Fri;13; 23.8; 25.9;  5;185;  7;  0; 43;  0;  0.1;  3;0.0;1013; 33;     0; 1;   0;
30.06.2023;Fri;14; 24.2; 26.2;  5;190;  7;  0; 45;  0;  0.2;  5;0.0;1014; 35;     0; 1;   0;
30.06.2023;Fri;15; 24.6; 26.6;  5;195;  8;  0; 45;  0;  0.2;  5;0.0;1015; 35;     0; 1;   0;
30.06.2023;Fri;16; 25.0; 27.0;  6;200;  8;  0; 45;  0;  0.2;  5;0.0;1016; 35;     0; 1;   0;
30.06.2023;Fri;17; 25.4; 27.4;  6;205;  8;  0; 45;  0;  0.2;  5;0.0;1017; 35;     0; 1;   0;
30.06.2023;Fri;18; 25.8; 27.8;  6;210;  9;  0; 45;  0;  0.2;  5;0.0;1018; 35;     0; 1;   0;
30.06.2023;Fri;19; 26.2; 28.2;  6;215;  9;  0; 45;  0;  0.2;  5;0.0;1019; 35;     0; 1;   0;
30.06.2023;Fri;20; 26.6; 28.6;  6;220; 11;  0; 45;  0;  0.2;  5;0.0;1020; 35;     0; 1;   0;
30.06.2023;Fri;21; 27.0; 29.0;  6;225; 15;  0; 45;  0;  0.2;  5;0.0;1021; 35;     0; 1;   0;
30.06.2023;Fri;22; 27.4; 29.4;  6;230; 19;  0; 45;  0;  0.2;  5;0.0;1022; 35;     0; 1;   0;
30.06.2023;Fri;23; 27.8; 29.8;  6;235; 13;  0; 45;  0;  0.2;  5;0.0;1023; 35;     0; 1;   0;
01.07.2023;Sat;00; 28.2; 30.2;  6;240; 17;  0; 45;  0;  0.2;  5;0.0;1024; 35;     0; 1;   0;
01.07.2023;Sat;01; 28.6; 30.6;  6;245; 11;  0; 45;  0;  0.2;  5;0.0;1025; 35;     0; 1;   0;
01.07.2023;Sat;02; 29.0; 31.0;  7;250; 15;  0; 45;  0;  0.2;  5;0.0;1026; 35;     0; 1;   0;
01.07.2023;Sat;03; 29.4; 31.4;  7;255; 19;  0; 45;  0;  0.2;  5;0.0;1027; 35;     0; 1;   0;
01.07.2023;Sat;04; 29.8; 31.8;  7;260; 13;  0; 45;  0;  0.2;  5;0.0;1028; 35;     0; 1;   0;
01.07.2023;Sat;05; 30.2; 32.2;  7;265; 17;  0; 45;  0;  0.2;  5;0.0;1029; 35;     0; 1;   0;
01.07.2023;Sat;06; 30.6; 32.6;  7;270; 11;  0; 45;  0;  0.2;  5;0.0;1030; 35;     0; 1;   0;
01.07.2023;Sat;07; 31.0; 33.0;  7;275; 15;  0; 45;  0;  0.2;  5;0.0;1031; 35;     0; 1;   0;
01.07.2023;Sat;08; 31.4; 33.4;  7;280; 19;  0; 45;  0;  0.2;  5;0.0;1032; 35;     0; 1;   0;
01.07.2023;Sat;09; 31.8; 33.8;  7;285; 13;  0; 45;  0;  0.2;  5;0.0;1033; 35;     0; 1;   0;
01.07.2023;Sat;10; 32.2; 34.2;  7;290; 17;  0; 45;  0;  0.2;  5;0.0;1034; 35;     0; 1;   0;
01.07.2023;Sat;11; 32.6; 34.6;  7;295; 11;  0; 45;  0;  0.2;  5;0.0;1035; 35;     0; 1;   0;
01.07.2023;Sat;12; 33.0; 35.0;  8;300; 15;  0; 45;  0;  0.2;  5;0.0;1036; 35;     0; 1;   0;
01.07.2023;Sat;13; 33.4; 35.4;  8;305; 19;  0; 45;  0;  0.2;  5;0.0;1037; 35;     0; 1;   0;

</station>"#;

struct AppState {
    weather_response: CachedResponse,
}

struct CachedResponse {
    time: SystemTime,
    response: String,
}

impl Default for CachedResponse {
    fn default() -> Self {
        Self {
            time: SystemTime::now(),
            response: String::new(),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            weather_response: CachedResponse::default(),
        }
    }
}

#[derive(Debug)]
struct WeatherReport {
    longitude: f64,
    latitude: f64,
    utc_offset: i64,
    daily: DailyData,
    hourly: HourlyData,
}

#[derive(Debug)]
struct DailyData {
    sundata: SunData,
}

#[derive(Debug)]
struct SunData {
    sunrise_time: DateTime<Utc>,
    sunset_time: DateTime<Utc>,
}

#[derive(Debug)]
struct HourlyData {
    data: Vec<HourlyDetails>,
}

#[derive(Debug)]
struct HourlyDetails {
    time: DateTime<Utc>,
    temperature: f64,
    apparent_temperature: f64,
    wind_speed: f64,
    wind_bearing: f64,
    wind_gust: f64,
    cloud_cover: f64,
    precip_intensity: f64,
    precip_probability: f64,
    pressure: f64,
    humidity: f64,
    uv_index: f64,
}

#[derive(Debug, Deserialize)]
struct ForecastQuery {
    user: String,
    coord: String,
    asl: u32,
    format: u32,
    new_api: u32,
}

#[tokio::main]
async fn main() {
    let app_state = Arc::new(RwLock::new(AppState::default()));

    // build our application with a single route
    let app = Router::new()
        .route("/forecast/", get(forecast_handler))
        .layer(Extension(app_state));
    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:6066".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[debug_handler]
async fn forecast_handler(
    Extension(state): Extension<Arc<RwLock<AppState>>>,
    Query(query): Query<ForecastQuery>,
) -> Response {
    println!("{:#?}", query);
    let body = create_response(state).await;
    let mut res = body.into_response();
    res.headers_mut()
        .insert("Vary", HeaderValue::from_static("Accept-Encoding"));
    res.headers_mut()
        .insert("Connection", HeaderValue::from_static("close"));
    res.headers_mut()
        .insert("Content-Type", HeaderValue::from_static("text/plain"));
    res
}

async fn create_response(state: Arc<RwLock<AppState>>) -> String {
    {
        let new_sys_time = SystemTime::now();
        let chached_state = state.read().unwrap();

        let difference = new_sys_time
            .duration_since(chached_state.weather_response.time)
            .expect("Clock may have gone backwards");
        if !chached_state.weather_response.response.is_empty() && difference.as_secs() < 5400 {
            return chached_state.weather_response.response.clone();
        }
    }

    let res = get_weather_from_provider("40", "50").await;

    {
        let state_mut = state.write();
        let new_response = CachedResponse {
            time: SystemTime::now(),
            response: res.clone(),
        };
        state_mut.unwrap().weather_response = new_response;
    }
    res
}

async fn get_weather_from_provider(lat: &str, long: &str) -> String {
    //https://api.pirateweather.net/forecast/[apikey]/[latitude],[longitude]
    //let url = "https://api.pirateweather.net/forecast/".to_owned() + API_KEY + "/" + lat + "," + long;
    use std::env;

    let key = "PIRATEWEATHER_API_KEY";
    match env::var(key) {
        Ok(val) => println!("{key}: {val:?}"),
        Err(e) => println!("couldn't interpret {key}: {e}"),
    }

    let asl = 8;
    let weather_report = generate_test_report();
    let mut csv = String::new();
    csv += "<mb_metadata>\n";
    csv += "id;name;longitude;latitude;height (m.asl.);country;timezone;utc-timedifference;sunrise;sunset;\n";
    csv += "local date;weekday;local time;temperature(C);feeledTemperature(C);windspeed(km/h);winddirection(degr);wind gust(km/h);low clouds(%);medium clouds(%);high clouds(%);precipitation(mm);probability of Precip(%);snowFraction;sea level pressure(hPa);relative humidity(%);CAPE;picto-code;radiation (W/m2);\n";
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

    let sunrise_time = weather_report
        .daily
        .sundata
        .sunrise_time
        .format("%H:%M")
        .to_string();
    let sunset_time = weather_report
        .daily
        .sundata
        .sunset_time
        .format("%H:%M")
        .to_string();

    csv += &format!(
        ";{};{:.2}°{};{:.2}°{};{};{};CEST;UTC{:+.1};{};{};\n",
        CITY_NAME,
        lon,
        ew,
        lat,
        ns,
        asl,
        COUNTRY_NAME,
        weather_report.utc_offset,
        sunrise_time,
        sunset_time
    );
    for hourly in &weather_report.hourly.data {
        let icon_id = 1; //TODO get actual icon

        write!(&mut csv, "{};{:5.1};{:5.1};{:3.0};{:3.0};{:3.0};{:3.0};{:3.0};{:3.0};{:5.1};{:3.0};{:3.1};{:4.0};{:3.0};{};{};{};\n",
    hourly.time.format("%d.%m.%Y;%a;%H").to_string(),
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
    (hourly.uv_index * 100.0) as i64).unwrap();

        // csv += &format!("{:%d.%m.%Y;%a;%H};", hourly.time);
        // csv += &format!("{:.1};", hourly.temperature);
        // csv += &format!("{:.1};", hourly.apparent_temperature);
        // csv += &format!("{:.0};", hourly.wind_speed);
        // csv += &format!("{:.0};", hourly.wind_bearing);
        // csv += &format!("{:.0};", hourly.wind_gust);
        // csv += &format!("{:.0};", 0.0);
        // csv += &format!("{:.0};", hourly.cloud_cover * 100.0);
        // csv += &format!("{:.0};", 0.0);
        // csv += &format!("{:.1};", hourly.precip_intensity);
        // csv += &format!("{:.0};", hourly.precip_probability);
        // csv += &format!("{:.1};", 0.0);
        // csv += &format!("{:.0};", hourly.pressure);
        // csv += &format!("{:.0};", hourly.humidity * 100.0);
        // csv += &format!("{:6};", 0);
        // csv += &format!("{};", icon_id);
        // csv += &format!("{:.0};\n", hourly.uv_index * 100.0);
    }

    csv += "</station>\n";

    print!("{csv}");
    TEST_DATA_CSV.to_owned()
}

fn generate_test_report() -> WeatherReport {
    let mut raw_data = Vec::new();

    for hour in 8..24 {
        raw_data.push((
            2023, 7, 1, hour, 25.0, 27.0, 5.0, 180.0, 7.0, 0.45, 0.2, 5.0, 1012.0, 0.35, 0.0,
        ));
    }

    for hour in 0..8 {
        raw_data.push((
            2023, 7, 2, hour, 25.0, 27.0, 5.0, 180.0, 7.0, 0.45, 0.2, 5.0, 1012.0, 0.35, 0.0,
        ));
    }

    let hourly_data: Vec<HourlyDetails> = raw_data
        .iter()
        .map(|data| HourlyDetails {
            time: Utc.ymd(data.0, data.1, data.2).and_hms(data.3, 0, 0),
            temperature: data.4,
            apparent_temperature: data.5,
            wind_speed: data.6,
            wind_bearing: data.7,
            wind_gust: data.8,
            cloud_cover: data.9,
            precip_intensity: data.10,
            precip_probability: data.11,
            pressure: data.12,
            humidity: data.13,
            uv_index: data.14,
        })
        .collect();

    let weather_report = WeatherReport {
        longitude: 40.0122,
        latitude: 65.002,
        utc_offset: 2,
        daily: DailyData {
            sundata: SunData {
                sunrise_time: Utc.with_ymd_and_hms(2023, 7, 1, 6, 0, 0).unwrap(),
                sunset_time: Utc.with_ymd_and_hms(2023, 7, 1, 22, 0, 0).unwrap(),
            },
        },
        hourly: HourlyData { data: Vec::new() },
    };
    weather_report
}
