use axum::http::HeaderValue;
use axum::{
    extract::Extension, extract::Query, http::StatusCode, response::IntoResponse,
    response::Response, routing::get, Router,
};
use serde::Deserialize;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

mod weather;

const CACHE_INVALIDATION_TIME: u64 = 3600;

#[derive(Default)]
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

async fn forecast_handler(
    Extension(state): Extension<Arc<RwLock<AppState>>>,
    Query(query): Query<ForecastQuery>,
) -> Response {
    println!(
        "New query received from lox {} with params: coord={}, asl={}, format={}, new_api={}",
        query.user, query.coord, query.asl, query.format, query.new_api
    );
    match create_response(state, query).await {
        Ok(body) => {
            let mut res = body.into_response();
            res.headers_mut()
                .insert("Vary", HeaderValue::from_static("Accept-Encoding"));
            res.headers_mut()
                .insert("Connection", HeaderValue::from_static("close"));
            res.headers_mut()
                .insert("Content-Type", HeaderValue::from_static("text/plain"));
            res
        }
        Err(e) => {
            eprintln!("Failed to create a valid response: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Something went wrong: {}", e),
            )
                .into_response()
        }
    }
}

async fn create_response(
    state: Arc<RwLock<AppState>>,
    query: ForecastQuery,
) -> std::result::Result<String, Box<dyn std::error::Error>> {
    {
        let new_sys_time = SystemTime::now();
        let chached_state = state.read().map_err(|_| "Could not acquire read lock")?;

        let difference = new_sys_time.duration_since(chached_state.weather_response.time)?;

        if !chached_state.weather_response.response.is_empty()
            && difference.as_secs() < CACHE_INVALIDATION_TIME
        {
            return Ok(chached_state.weather_response.response.clone());
        }
    }

    //Get the coordinates from query. Loxone gives them in reverse for some reason
    let split: Vec<&str> = query.coord.split(',').collect();

    let long: f64 = split
        .first()
        .ok_or("Failed to get longitude")?
        .parse()
        .map_err(|_| "Failed to parse longitude")?;

    let lat: f64 = split
        .get(1)
        .ok_or("Failed to get latitude")?
        .parse()
        .map_err(|_| "Failed to parse latitude")?;

    let options = weather::DataCreationOptions {
        latitude: lat,
        longitude: long,
        above_sea_level: query.asl,
    };
    match weather::create_weather_xml(options).await {
        Ok(val) => {
            let mut state_mut = state.write().map_err(|_| "Could not acquire write lock")?;
            let new_response = CachedResponse {
                time: SystemTime::now(),
                response: val.clone(),
            };
            state_mut.weather_response = new_response;

            Ok(val)
        }
        Err(err) => Err(err),
    }
}
