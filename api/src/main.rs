mod conversion;
mod deserializers;

use std::{collections::HashMap, sync::Arc, time::Duration};

use anyhow::anyhow;
use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
};
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime};
use serde::Deserialize;

use crate::conversion::calculate_no_of_martian_sol_elapsed;
use crate::deserializers::{
    i64_from_string, naivedate_from_string, naivetime_from_string, sole_from_string,
};

#[derive(Debug, PartialEq, Eq, Deserialize, Hash, Clone)]
pub struct Sole(i64);

impl From<i64> for Sole {
    fn from(value: i64) -> Self {
        Sole(value)
    }
}

#[derive(Debug, Deserialize, Clone)]
struct SoleData {
    #[allow(dead_code)]
    id: String,

    #[allow(dead_code)]
    #[serde(deserialize_with = "naivedate_from_string")]
    terrestrial_date: NaiveDate,

    #[serde(deserialize_with = "sole_from_string")]
    sol: Sole,

    #[serde(deserialize_with = "i64_from_string")]
    min_temp: Option<i64>,
    #[serde(deserialize_with = "i64_from_string")]
    max_temp: Option<i64>,

    #[serde(deserialize_with = "naivetime_from_string")]
    sunrise: NaiveTime,
    #[serde(deserialize_with = "naivetime_from_string")]
    sunset: NaiveTime,
}

struct InnerCachedSolesData {
    updated_at: chrono::DateTime<chrono::Utc>,
    data: HashMap<Sole, SoleData>,
}

struct CachedSolesData(tokio::sync::RwLock<InnerCachedSolesData>);

impl CachedSolesData {
    pub fn new(data: HashMap<Sole, SoleData>) -> Self {
        CachedSolesData(tokio::sync::RwLock::new(InnerCachedSolesData {
            updated_at: chrono::Utc::now(),
            data,
        }))
    }

    pub async fn get_data_for_sol(&self, sol: impl Into<Sole>) -> Option<SoleData> {
        self.0.read().await.data.get(&sol.into()).cloned()
    }

    pub async fn update(&self, data: HashMap<Sole, SoleData>) {
        self.0.write().await.data = data;
        self.0.write().await.updated_at = chrono::Utc::now();
    }
}

struct SharedState {
    cached_soles_data: CachedSolesData,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let soles_data = fetch_soles_data()
        .await
        .map_err(|err| anyhow::anyhow!("Unable to fetch soles data: {err}"))?;

    let shared_state = Arc::new(SharedState {
        cached_soles_data: CachedSolesData::new(soles_data),
    });

    let shared_state_clone = shared_state.clone();
    // Starts background thread that updates cached data once an hour
    let updater_handle = tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_hours(1)).await;
            tracing::info!("Updating soles data...");
            match fetch_soles_data().await {
                Ok(data) => {
                    shared_state_clone.cached_soles_data.update(data).await;
                    tracing::info!("Updated soles data!");
                }
                Err(err) => {
                    tracing::error!(
                        "Unable to fetch soles data. Trying again in 1 hour. Err: {err}"
                    );
                }
            }
        }
    });

    // build our application with a single route
    let app = Router::new()
        .route("/", get(hello))
        .route("/weather", get(weather))
        .with_state(shared_state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    let server_handle = tokio::spawn(async move {
        tracing::info!("Starting server...");
        axum::serve(listener, app).await.unwrap();
    });

    match tokio::signal::ctrl_c().await {
        Ok(_) => {
            updater_handle.abort();
            tracing::info!("Shutting down server...")
        }
        Err(_) => {
            tracing::error!("Unable to listen for shutdown signal...")
        }
    }

    server_handle.abort();

    Ok(())
}

async fn hello() -> Html<&'static str> {
    Html(
        r"
        <h1>Hello!</h1>
        <section>
            <p>Weather api is available as /weather.</p>
            <p>Use /weather?date=[requested date].
            <br/>
            Valid formats for date are %Y-%m-%d (e.g. 2026-02-15) or rfc3339 (e.g. 2026-02-15T21:42:00%2B01:00 or 2026-02-15T20:42:00Z).
            </p>
        </section>",
    )
}

#[derive(Debug, Deserialize)]
struct WeatherQuery {
    date: Option<String>,
}

/// Handler that serves weather data for requested date
async fn weather(
    Query(params): Query<WeatherQuery>,
    State(state): State<Arc<SharedState>>,
) -> impl IntoResponse {
    if let Some(maybe_date) = params.date {
        let datetime = match parse_date_from_string(&maybe_date) {
            Ok(valid_datetime) => valid_datetime,
            Err(err) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "error": "INVALID_DATE_FORMAT",
                        "message": err.to_string() })),
                )
                    .into_response();
            }
        };

        let date_in_martian_sols = calculate_no_of_martian_sol_elapsed(datetime);

        match state
            .cached_soles_data
            .get_data_for_sol(date_in_martian_sols)
            .await
        {
            Some(data) => (
                StatusCode::OK,
                Json(serde_json::json!({
                    "martian_sol_day": data.sol.0.to_string(),

                    "min_temp": data.min_temp.map(|temp| temp.to_string()).unwrap_or("N/A".to_string()),
                    "max_temp": data.max_temp.map(|temp| temp.to_string()).unwrap_or("N/A".to_string()),

                    "sunrise": data.sunrise,
                    "sunset": data.sunset
                })),
            )
                .into_response(),
            None => (
                StatusCode::NO_CONTENT,
                Json(serde_json::json!({
                    "message": "No data found for date"
                })),
            )
                .into_response(),
        }
    } else {
        (StatusCode::OK, Json(serde_json::json!({
            "message": "Send request with query parameter ?date=<requested date>. Allowed formats are %Y-%m-%d and rfc3339."
        }))).into_response()
    }
}

fn parse_date_from_string(maybe_date: &str) -> anyhow::Result<chrono::DateTime<chrono::Utc>> {
    tracing::info!("Parsing date: {maybe_date}");
    let naive_date = NaiveDate::parse_from_str(maybe_date, "%Y-%m-%d");
    let rfc3339_date = DateTime::parse_from_rfc3339(maybe_date);

    match (naive_date, rfc3339_date) {
        (Ok(naive_date), _) => Ok(NaiveDateTime::new(naive_date, NaiveTime::default()).and_utc()),
        (_, Ok(rfc3339_date)) => Ok(rfc3339_date.to_utc()),
        (naive_date_err, rfc3339_date_err) => {
            tracing::error!(
                "naive_date_err: {:#?}, rfc3339_date_err: {:#?}",
                naive_date_err,
                rfc3339_date_err
            );
            Err(anyhow!(
                "Invalid format for date. Allowed formats are %Y-%m-%d and rfc3339."
            ))
        }
    }
}

#[derive(Debug, Deserialize)]
struct NasaData {
    soles: Vec<SoleData>,
}

async fn fetch_soles_data() -> anyhow::Result<HashMap<Sole, SoleData>> {
    let res = reqwest::get(
        "https://mars.nasa.gov/rss/api/?feed=weather&feedtype=json&ver=1.0&category=msl",
    )
    .await?;

    let soles = match res.json::<NasaData>().await {
        Ok(data) => data.soles,
        Err(err) => {
            tracing::error!("Failed to fetch soles data: {}", err);
            return Err(anyhow!(err));
        }
    };

    Ok(soles.into_iter().fold(HashMap::new(), |mut acc, sole| {
        acc.insert(sole.sol.clone(), sole);
        acc
    }))
}
