use std::cmp::Ordering;
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
impl<S> From<S> for Symbol
where
    S: Into<String>,
{
    fn from(s: S) -> Symbol {
        Symbol::new(s)
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
    time_series: HashMap<chrono::NaiveDate, TimeSeriesDay>,
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

pub enum TimePeriod {
    Month,
    Year,
    AllTime,
}

#[derive(Debug)]
pub struct EquitySummary {
    latest_price: f64,
    earliest_price: f64,
    max_price: f64,
    min_price: f64,
}
pub fn summary_for_equity(
    symbol: Symbol,
    time_period: TimePeriod,
) -> Result<EquitySummary, ApiError> {
    let now = chrono::Utc::now();
    let today = now.date().naive_local();

    let time_series = get_time_series_daily(&CLIENT, symbol, DailyOutputSize::Full)?.time_series;

    let time_series: HashMap<_, _> = time_series
        .into_iter()
        .filter(|(date, data)| match time_period {
            TimePeriod::Month => *date + chrono::Duration::days(30) >= today,
            TimePeriod::Year => *date + chrono::Duration::days(365) >= today,
            TimePeriod::AllTime => true,
        })
        .collect();

    Ok(EquitySummary {
        latest_price: time_series
            .iter()
            .max_by_key(|&(date, data)| date)
            .map(|(_date, data)| data.close)
            .unwrap(),
        earliest_price: time_series
            .iter()
            .min_by_key(|&(date, data)| date)
            .map(|(_date, data)| data.close)
            .unwrap(),
        max_price: time_series
            .values()
            .map(|data| data.high)
            .max_by(f64_ord_panic)
            .unwrap(),
        min_price: time_series
            .values()
            .map(|data| data.low)
            .min_by(f64_ord_panic)
            .unwrap(),
    })
}

fn f64_ord_panic(a: &f64, b: &f64) -> Ordering {
    if a > b {
        Ordering::Greater
    } else if a < b {
        Ordering::Less
    } else if a == b {
        Ordering::Equal
    } else {
        panic!("input must be comparable")
    }
}
