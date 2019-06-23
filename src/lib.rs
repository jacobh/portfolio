use std::collections::HashMap;
use std::env;
use std::ops::Deref;

use lazy_static::lazy_static;
use reqwest;
use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string;

lazy_static! {
    static ref CLIENT: reqwest::Client = reqwest::Client::new();
    static ref VANTAGE_API_KEY: String =
        env::var("VANTAGE_API_KEY").expect("`VANTAGE_API_KEY` environment variable must be set");
}

pub struct Symbol(String);
impl Symbol {
    pub fn new<S: Into<String>>(s: S) -> Symbol {
        Symbol(s.into())
    }
}

impl Deref for Symbol {
    type Target = str;
    fn deref(&self) -> &str {
        self.0.as_str()
    }
}

enum DailyOutputSize {
    Compact,
    Full,
}
impl DailyOutputSize {
    fn as_str(&self) -> &'static str {
        match self {
            DailyOutputSize::Compact => "compact",
            DailyOutputSize::Full => "full",
        }
    }
}

#[derive(Debug)]
pub enum ApiError {
    Reqwest(reqwest::Error),
}
impl From<reqwest::Error> for ApiError {
    fn from(error: reqwest::Error) -> ApiError {
        ApiError::Reqwest(error)
    }
}

#[derive(Debug, Deserialize)]
struct TimeSeriesDay {
    #[serde(
        rename = "1. open",
        deserialize_with = "deserialize_number_from_string"
    )]
    open: f64,
    #[serde(
        rename = "2. high",
        deserialize_with = "deserialize_number_from_string"
    )]
    high: f64,
    #[serde(rename = "3. low", deserialize_with = "deserialize_number_from_string")]
    low: f64,
    #[serde(
        rename = "4. close",
        deserialize_with = "deserialize_number_from_string"
    )]
    close: f64,
    #[serde(
        rename = "5. adjusted close",
        deserialize_with = "deserialize_number_from_string"
    )]
    adjusted_close: f64,
    #[serde(
        rename = "6. volume",
        deserialize_with = "deserialize_number_from_string"
    )]
    volume: f64,
    #[serde(
        rename = "7. dividend amount",
        deserialize_with = "deserialize_number_from_string"
    )]
    dividend_amount: f64,
    #[serde(
        rename = "8. split coefficient",
        deserialize_with = "deserialize_number_from_string"
    )]
    split_coefficient: f64,
}

#[derive(Debug, Deserialize)]
struct TimeSeriesDailyResponse {
    #[serde(rename = "Meta Data")]
    metadata: serde_json::Value,
    #[serde(rename = "Time Series (Daily)")]
    time_series: HashMap<String, TimeSeriesDay>,
}

fn get_time_series_daily(
    client: &reqwest::Client,
    symbol: Symbol,
    output_size: DailyOutputSize,
) -> Result<TimeSeriesDailyResponse, ApiError> {
    client
        .get("https://www.alphavantage.co/query")
        .query(&[
            ("function", "TIME_SERIES_DAILY_ADJUSTED"),
            ("symbol", &*symbol),
            ("apikey", &*VANTAGE_API_KEY),
            ("outputsize", output_size.as_str()),
        ])
        .send()
        .and_then(|resp| resp.error_for_status())
        .and_then(|mut resp| resp.json())
        .map_err(|err| err.into())
}

pub fn get_latest_price_for_equity(symbol: Symbol) -> Result<f64, ApiError> {
    let result = get_time_series_daily(&CLIENT, symbol, DailyOutputSize::Compact)?;

    Ok(result
        .time_series
        .iter()
        .max_by_key(|&(date, data)| date)
        .map(|(date, data)| data.close)
        .unwrap())
}
