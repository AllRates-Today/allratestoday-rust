//! # AllRatesToday Rust SDK
//!
//! Official Rust client for the [AllRatesToday](https://allratestoday.com) exchange rate API.
//! Provides access to real-time mid-market exchange rates for 160+ currencies.
//!
//! ## Quick Start
//!
//! ```no_run
//! use allratestoday::AllRatesToday;
//!
//! let client = AllRatesToday::new("art_live_your_api_key");
//!
//! // Get latest rates for USD against EUR and GBP
//! let rates = client.latest("USD", Some(&["EUR", "GBP"])).unwrap();
//! println!("{:?}", rates);
//!
//! // Convert 100 USD to EUR
//! let result = client.convert("USD", "EUR", 100.0).unwrap();
//! println!("{:?}", result);
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors that can occur when using the AllRatesToday SDK.
#[derive(Debug)]
pub enum AllRatesTodayError {
    /// An HTTP-level error from the underlying transport (network failure,
    /// timeout, TLS error, etc.).
    HttpError(reqwest::Error),
    /// The API returned an error response (non-2xx status code).
    ApiError {
        /// HTTP status code returned by the API.
        status: u16,
        /// Human-readable error message from the API response, if available.
        message: String,
    },
    /// Failed to parse the API response body.
    ParseError(String),
}

impl fmt::Display for AllRatesTodayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AllRatesTodayError::HttpError(e) => write!(f, "HTTP error: {e}"),
            AllRatesTodayError::ApiError { status, message } => {
                write!(f, "API error (HTTP {status}): {message}")
            }
            AllRatesTodayError::ParseError(msg) => write!(f, "Parse error: {msg}"),
        }
    }
}

impl std::error::Error for AllRatesTodayError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AllRatesTodayError::HttpError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<reqwest::Error> for AllRatesTodayError {
    fn from(err: reqwest::Error) -> Self {
        AllRatesTodayError::HttpError(err)
    }
}

impl From<serde_json::Error> for AllRatesTodayError {
    fn from(err: serde_json::Error) -> Self {
        AllRatesTodayError::ParseError(err.to_string())
    }
}

/// Convenience alias used throughout the crate.
pub type Result<T> = std::result::Result<T, AllRatesTodayError>;

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

/// Response from the `/v1/latest` endpoint.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LatestRatesResponse {
    /// Whether the request was successful.
    pub success: Option<bool>,
    /// Base currency code.
    pub base: Option<String>,
    /// Date the rates apply to (YYYY-MM-DD).
    pub date: Option<String>,
    /// Map of currency codes to their exchange rates relative to the base.
    pub rates: Option<HashMap<String, f64>>,
}

/// Response from the `/v1/convert` endpoint.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConvertResponse {
    /// Whether the request was successful.
    pub success: Option<bool>,
    /// Source currency code.
    pub from: Option<String>,
    /// Target currency code.
    pub to: Option<String>,
    /// Amount that was converted.
    pub amount: Option<f64>,
    /// Converted result.
    pub result: Option<f64>,
    /// Exchange rate used for the conversion.
    pub rate: Option<f64>,
}

/// Response from the `/v1/historical` endpoint.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HistoricalRatesResponse {
    /// Whether the request was successful.
    pub success: Option<bool>,
    /// Base currency code.
    pub base: Option<String>,
    /// The historical date (YYYY-MM-DD).
    pub date: Option<String>,
    /// Map of currency codes to their exchange rates on that date.
    pub rates: Option<HashMap<String, f64>>,
}

/// Response from the `/v1/timeseries` endpoint.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TimeSeriesResponse {
    /// Whether the request was successful.
    pub success: Option<bool>,
    /// Base currency code.
    pub base: Option<String>,
    /// Start date of the time series (YYYY-MM-DD).
    pub start_date: Option<String>,
    /// End date of the time series (YYYY-MM-DD).
    pub end_date: Option<String>,
    /// Map of dates to their respective rate maps.
    pub rates: Option<HashMap<String, HashMap<String, f64>>>,
}

/// A single currency entry returned by the `/v1/symbols` endpoint.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CurrencySymbol {
    /// Three-letter currency code (e.g. "USD").
    pub code: Option<String>,
    /// Human-readable currency name (e.g. "United States Dollar").
    pub name: Option<String>,
}

/// Response from the `/v1/symbols` endpoint.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SymbolsResponse {
    /// Whether the request was successful.
    pub success: Option<bool>,
    /// Map of currency codes to currency details.
    pub symbols: Option<HashMap<String, serde_json::Value>>,
}

/// Response from the `/v1/rate` endpoint (single pair).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RateResponse {
    /// Whether the request was successful.
    pub success: Option<bool>,
    /// Source currency code.
    pub from: Option<String>,
    /// Target currency code.
    pub to: Option<String>,
    /// The exchange rate.
    pub rate: Option<f64>,
    /// Date the rate applies to.
    pub date: Option<String>,
}

/// Response from the `/v1/historical-rates` endpoint (preset periods).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HistoricalRateSeriesResponse {
    /// Whether the request was successful.
    pub success: Option<bool>,
    /// Source currency code.
    pub source: Option<String>,
    /// Target currency code.
    pub target: Option<String>,
    /// The requested period (e.g. "7d").
    pub period: Option<String>,
    /// List of historical rate data points.
    pub rates: Option<Vec<HistoricalRatePoint>>,
}

/// A single data point within a historical rate series.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HistoricalRatePoint {
    /// Date of this data point (YYYY-MM-DD).
    pub date: Option<String>,
    /// Exchange rate on this date.
    pub rate: Option<f64>,
}

// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------

/// The main client for interacting with the AllRatesToday API.
///
/// Create an instance with [`AllRatesToday::new`] and then call the various
/// methods to query exchange rate data.
///
/// # Example
///
/// ```no_run
/// use allratestoday::AllRatesToday;
///
/// let client = AllRatesToday::new("art_live_your_api_key");
/// let rates = client.latest("USD", None).unwrap();
/// ```
pub struct AllRatesToday {
    api_key: String,
    base_url: String,
    client: reqwest::blocking::Client,
}

impl AllRatesToday {
    /// The default base URL for the AllRatesToday API.
    const DEFAULT_BASE_URL: &'static str = "https://allratestoday.com";

    /// Create a new client with the given API key.
    ///
    /// API keys follow the format `art_live_...` and can be obtained from
    /// <https://allratestoday.com>.
    ///
    /// # Arguments
    ///
    /// * `api_key` - Your AllRatesToday API key.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: Self::DEFAULT_BASE_URL.to_string(),
            client: reqwest::blocking::Client::new(),
        }
    }

    /// Create a new client with a custom base URL (useful for testing).
    ///
    /// # Arguments
    ///
    /// * `api_key`  - Your AllRatesToday API key.
    /// * `base_url` - Custom base URL for the API.
    pub fn with_base_url(api_key: impl Into<String>, base_url: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: base_url.into(),
            client: reqwest::blocking::Client::new(),
        }
    }

    // -- internal helpers ---------------------------------------------------

    /// Build a GET request with the Authorization header already set.
    fn request(&self, path: &str) -> reqwest::blocking::RequestBuilder {
        let url = format!("{}{}", self.base_url, path);
        self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
    }

    /// Execute a request and return the response body as a parsed type.
    fn execute<T: serde::de::DeserializeOwned>(
        &self,
        builder: reqwest::blocking::RequestBuilder,
    ) -> Result<T> {
        let response = builder.send()?;
        let status = response.status();

        if !status.is_success() {
            let body = response.text().unwrap_or_default();
            return Err(AllRatesTodayError::ApiError {
                status: status.as_u16(),
                message: body,
            });
        }

        let body = response.text()?;
        let parsed: T = serde_json::from_str(&body)?;
        Ok(parsed)
    }

    // -- public API ---------------------------------------------------------

    /// Get the latest exchange rates for the given base currency.
    ///
    /// # Arguments
    ///
    /// * `base`    - The base currency code (e.g. `"USD"`).
    /// * `symbols` - Optional slice of target currency codes to restrict the
    ///               response. Pass `None` to receive all available currencies.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use allratestoday::AllRatesToday;
    /// let client = AllRatesToday::new("art_live_your_api_key");
    /// let rates = client.latest("USD", Some(&["EUR", "GBP"])).unwrap();
    /// println!("Rates: {:?}", rates.rates);
    /// ```
    pub fn latest(&self, base: &str, symbols: Option<&[&str]>) -> Result<LatestRatesResponse> {
        let mut req = self.request("/v1/latest").query(&[("base", base)]);
        if let Some(syms) = symbols {
            req = req.query(&[("symbols", syms.join(","))]);
        }
        self.execute(req)
    }

    /// Convert an amount from one currency to another.
    ///
    /// # Arguments
    ///
    /// * `from`   - Source currency code.
    /// * `to`     - Target currency code.
    /// * `amount` - The amount to convert.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use allratestoday::AllRatesToday;
    /// let client = AllRatesToday::new("art_live_your_api_key");
    /// let result = client.convert("USD", "EUR", 100.0).unwrap();
    /// println!("Converted: {:?}", result.result);
    /// ```
    pub fn convert(&self, from: &str, to: &str, amount: f64) -> Result<ConvertResponse> {
        let req = self
            .request("/v1/convert")
            .query(&[("from", from), ("to", to)])
            .query(&[("amount", &amount.to_string())]);
        self.execute(req)
    }

    /// Get historical exchange rates for a specific date.
    ///
    /// # Arguments
    ///
    /// * `date`    - The date in `YYYY-MM-DD` format.
    /// * `base`    - The base currency code.
    /// * `symbols` - Optional slice of target currency codes.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use allratestoday::AllRatesToday;
    /// let client = AllRatesToday::new("art_live_your_api_key");
    /// let rates = client.for_date("2025-01-15", "USD", Some(&["EUR"])).unwrap();
    /// println!("Historical: {:?}", rates.rates);
    /// ```
    pub fn for_date(
        &self,
        date: &str,
        base: &str,
        symbols: Option<&[&str]>,
    ) -> Result<HistoricalRatesResponse> {
        let mut req = self
            .request("/v1/historical")
            .query(&[("date", date), ("base", base)]);
        if let Some(syms) = symbols {
            req = req.query(&[("symbols", syms.join(","))]);
        }
        self.execute(req)
    }

    /// Get a time series of exchange rates between two dates.
    ///
    /// # Arguments
    ///
    /// * `start_date` - Start date in `YYYY-MM-DD` format.
    /// * `end_date`   - End date in `YYYY-MM-DD` format.
    /// * `base`       - The base currency code.
    /// * `symbols`    - Optional slice of target currency codes.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use allratestoday::AllRatesToday;
    /// let client = AllRatesToday::new("art_live_your_api_key");
    /// let series = client.time_series(
    ///     "2025-01-01", "2025-01-31", "USD", Some(&["EUR", "GBP"]),
    /// ).unwrap();
    /// println!("Time series: {:?}", series.rates);
    /// ```
    pub fn time_series(
        &self,
        start_date: &str,
        end_date: &str,
        base: &str,
        symbols: Option<&[&str]>,
    ) -> Result<TimeSeriesResponse> {
        let mut req = self.request("/v1/timeseries").query(&[
            ("start_date", start_date),
            ("end_date", end_date),
            ("base", base),
        ]);
        if let Some(syms) = symbols {
            req = req.query(&[("symbols", syms.join(","))]);
        }
        self.execute(req)
    }

    /// List all supported currency symbols.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use allratestoday::AllRatesToday;
    /// let client = AllRatesToday::new("art_live_your_api_key");
    /// let symbols = client.symbols().unwrap();
    /// println!("Symbols: {:?}", symbols.symbols);
    /// ```
    pub fn symbols(&self) -> Result<SymbolsResponse> {
        let req = self.request("/v1/symbols");
        self.execute(req)
    }

    /// Get the exchange rate for a single currency pair.
    ///
    /// # Arguments
    ///
    /// * `from` - Source currency code.
    /// * `to`   - Target currency code.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use allratestoday::AllRatesToday;
    /// let client = AllRatesToday::new("art_live_your_api_key");
    /// let rate = client.get_rate("USD", "EUR").unwrap();
    /// println!("USD/EUR rate: {:?}", rate.rate);
    /// ```
    pub fn get_rate(&self, from: &str, to: &str) -> Result<RateResponse> {
        let req = self
            .request("/v1/rate")
            .query(&[("from", from), ("to", to)]);
        self.execute(req)
    }

    /// Get historical rates for a currency pair over a preset period.
    ///
    /// Supported periods: `"1d"`, `"7d"`, `"30d"`, `"1y"`.
    ///
    /// # Arguments
    ///
    /// * `source` - Source currency code.
    /// * `target` - Target currency code.
    /// * `period` - One of the preset period strings.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use allratestoday::AllRatesToday;
    /// let client = AllRatesToday::new("art_live_your_api_key");
    /// let history = client.get_historical_rates("USD", "EUR", "30d").unwrap();
    /// for point in history.rates.unwrap_or_default() {
    ///     println!("{}: {:?}", point.date.unwrap_or_default(), point.rate);
    /// }
    /// ```
    pub fn get_historical_rates(
        &self,
        source: &str,
        target: &str,
        period: &str,
    ) -> Result<HistoricalRateSeriesResponse> {
        let req = self
            .request("/v1/historical-rates")
            .query(&[("source", source), ("target", target), ("period", period)]);
        self.execute(req)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_stores_api_key() {
        let client = AllRatesToday::new("art_live_test_key_123");
        assert_eq!(client.api_key, "art_live_test_key_123");
        assert_eq!(client.base_url, "https://allratestoday.com");
    }

    #[test]
    fn client_custom_base_url() {
        let client = AllRatesToday::with_base_url("art_live_key", "https://custom.example.com");
        assert_eq!(client.base_url, "https://custom.example.com");
    }

    #[test]
    fn parse_latest_response() {
        let json = r#"{
            "success": true,
            "base": "USD",
            "date": "2025-06-01",
            "rates": { "EUR": 0.92, "GBP": 0.79 }
        }"#;
        let resp: LatestRatesResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.success, Some(true));
        assert_eq!(resp.base.as_deref(), Some("USD"));
        let rates = resp.rates.unwrap();
        assert!((rates["EUR"] - 0.92).abs() < f64::EPSILON);
    }

    #[test]
    fn parse_convert_response() {
        let json = r#"{
            "success": true,
            "from": "USD",
            "to": "EUR",
            "amount": 100.0,
            "result": 92.0,
            "rate": 0.92
        }"#;
        let resp: ConvertResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.success, Some(true));
        assert_eq!(resp.result, Some(92.0));
    }

    #[test]
    fn parse_symbols_response() {
        let json = r#"{
            "success": true,
            "symbols": {
                "USD": { "code": "USD", "name": "United States Dollar" },
                "EUR": { "code": "EUR", "name": "Euro" }
            }
        }"#;
        let resp: SymbolsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.success, Some(true));
        assert!(resp.symbols.unwrap().contains_key("USD"));
    }

    #[test]
    fn parse_time_series_response() {
        let json = r#"{
            "success": true,
            "base": "USD",
            "start_date": "2025-01-01",
            "end_date": "2025-01-03",
            "rates": {
                "2025-01-01": { "EUR": 0.91 },
                "2025-01-02": { "EUR": 0.92 },
                "2025-01-03": { "EUR": 0.93 }
            }
        }"#;
        let resp: TimeSeriesResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.success, Some(true));
        assert_eq!(resp.rates.unwrap().len(), 3);
    }

    #[test]
    fn parse_rate_response() {
        let json = r#"{
            "success": true,
            "from": "USD",
            "to": "EUR",
            "rate": 0.92,
            "date": "2025-06-01"
        }"#;
        let resp: RateResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.rate, Some(0.92));
    }

    #[test]
    fn parse_historical_rate_series_response() {
        let json = r#"{
            "success": true,
            "source": "USD",
            "target": "EUR",
            "period": "7d",
            "rates": [
                { "date": "2025-05-25", "rate": 0.91 },
                { "date": "2025-05-26", "rate": 0.92 }
            ]
        }"#;
        let resp: HistoricalRateSeriesResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.period.as_deref(), Some("7d"));
        assert_eq!(resp.rates.unwrap().len(), 2);
    }

    #[test]
    fn error_display() {
        let err = AllRatesTodayError::ApiError {
            status: 401,
            message: "Unauthorized".to_string(),
        };
        assert_eq!(err.to_string(), "API error (HTTP 401): Unauthorized");

        let err = AllRatesTodayError::ParseError("bad json".to_string());
        assert_eq!(err.to_string(), "Parse error: bad json");
    }
}
