//! # AllRatesToday Rust SDK
//!
//! Official Rust client for the [AllRatesToday](https://allratestoday.com) exchange rate API:
//! real-time mid-market rates for 160+ currencies, conversion, up to a year of history (paid
//! plans), and the official rates published by 121 central banks and 3 tax authorities.
//!
//! Get a free API key at <https://allratestoday.com/register>.
//! [`AllRatesToday::symbols`], [`AllRatesToday::official_rates`] and
//! [`AllRatesToday::official_sources`] work without a key.
//!
//! History and past-date rates ([`AllRatesToday::convert_on_date`], [`AllRatesToday::for_date`],
//! [`AllRatesToday::time_series`], [`AllRatesToday::get_historical_rates`]) need a paid plan;
//! free and trial keys get [`AllRatesTodayError::ApiError`] with status 403 and code
//! `historical_requires_paid_plan`. The keyed API sends no CORS headers, so use it server-side.
//!
//! ## Quick Start
//!
//! ```no_run
//! use allratestoday::AllRatesToday;
//!
//! let client = AllRatesToday::new("art_live_your_api_key");
//!
//! // Latest rates for USD against EUR and GBP
//! let rates = client.latest("USD", Some(&["EUR", "GBP"])).unwrap();
//! println!("{:?}", rates.rates);
//!
//! // Convert 100 USD to EUR
//! let result = client.convert("USD", "EUR", 100.0).unwrap();
//! println!("{:?}", result.result);
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
        /// The response body (usually `{"error":"..."}`).
        message: String,
    },
    /// Failed to parse the API response body.
    ParseError(String),
    /// The call was rejected before any request was sent: missing API key,
    /// missing symbols, a malformed or out-of-range date, or an invalid
    /// official-source code.
    InvalidArgument(String),
}

impl fmt::Display for AllRatesTodayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AllRatesTodayError::HttpError(e) => write!(f, "HTTP error: {e}"),
            AllRatesTodayError::ApiError { status, message } => {
                write!(f, "API error (HTTP {status}): {message}")
            }
            AllRatesTodayError::ParseError(msg) => write!(f, "Parse error: {msg}"),
            AllRatesTodayError::InvalidArgument(msg) => write!(f, "Invalid argument: {msg}"),
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

fn invalid<T>(msg: impl Into<String>) -> Result<T> {
    Err(AllRatesTodayError::InvalidArgument(msg.into()))
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

/// One element of the list returned by `GET /api/v1/rates`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Rate {
    /// The exchange rate.
    pub rate: f64,
    /// Base currency code.
    pub source: String,
    /// Quote currency code.
    pub target: String,
    /// Observation time, e.g. `2026-10-16T11:31:01+0000`.
    pub time: Option<String>,
}

/// Returned by [`AllRatesToday::latest`].
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LatestRatesResponse {
    /// Always `Some(true)`; failures are returned as `Err`.
    pub success: Option<bool>,
    /// Base currency code.
    pub base: Option<String>,
    /// Observation time of the rates, e.g. `2026-10-16T11:31:01+0000`.
    pub date: Option<String>,
    /// Map of currency codes to their exchange rates relative to the base.
    pub rates: Option<HashMap<String, f64>>,
}

/// Returned by [`AllRatesToday::convert`] and [`AllRatesToday::convert_on_date`].
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConvertResponse {
    /// Always `Some(true)`; failures are returned as `Err`.
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
    /// The date (YYYY-MM-DD) for [`AllRatesToday::convert_on_date`]; `None` for current rates.
    pub date: Option<String>,
}

/// Returned by [`AllRatesToday::for_date`].
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HistoricalRatesResponse {
    /// Always `Some(true)`; failures are returned as `Err`.
    pub success: Option<bool>,
    /// Base currency code.
    pub base: Option<String>,
    /// The historical date (YYYY-MM-DD).
    pub date: Option<String>,
    /// Map of currency codes to their exchange rates on that date.
    pub rates: Option<HashMap<String, f64>>,
}

/// Returned by [`AllRatesToday::time_series`].
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TimeSeriesResponse {
    /// Always `Some(true)`; failures are returned as `Err`.
    pub success: Option<bool>,
    /// Base currency code.
    pub base: Option<String>,
    /// Start date of the time series (YYYY-MM-DD).
    pub start_date: Option<String>,
    /// End date of the time series (YYYY-MM-DD).
    pub end_date: Option<String>,
    /// Map of dates (YYYY-MM-DD) to currency-code -> rate maps.
    pub rates: Option<HashMap<String, HashMap<String, f64>>>,
}

/// A single currency returned by `GET /api/v1/symbols`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CurrencySymbol {
    /// Three-letter currency code (e.g. "USD").
    pub code: Option<String>,
    /// Human-readable currency name (e.g. "US Dollar").
    pub name: Option<String>,
    /// Display symbol (e.g. "$").
    pub symbol: Option<String>,
}

/// Returned by [`AllRatesToday::symbols`].
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SymbolsResponse {
    /// Always `Some(true)`; failures are returned as `Err`.
    pub success: Option<bool>,
    /// Map of currency codes to `{"code","name","symbol"}` objects.
    pub symbols: Option<HashMap<String, serde_json::Value>>,
    /// The currencies in the order the API returns them.
    pub currencies: Option<Vec<CurrencySymbol>>,
    /// Number of currencies.
    pub count: Option<u32>,
}

/// Returned by [`AllRatesToday::get_rate`].
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RateResponse {
    /// Always `Some(true)`; failures are returned as `Err`.
    pub success: Option<bool>,
    /// Source currency code.
    pub from: Option<String>,
    /// Target currency code.
    pub to: Option<String>,
    /// The exchange rate.
    pub rate: Option<f64>,
    /// Observation time, e.g. `2026-10-16T11:31:01+0000`.
    pub date: Option<String>,
}

/// Returned by [`AllRatesToday::get_historical_rates`].
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HistoricalRateSeriesResponse {
    /// Always `Some(true)`; failures are returned as `Err`.
    pub success: Option<bool>,
    /// Source currency code.
    pub source: Option<String>,
    /// Target currency code.
    pub target: Option<String>,
    /// The requested period (e.g. "7d").
    pub period: Option<String>,
    /// Data points, oldest first.
    pub rates: Option<Vec<HistoricalRatePoint>>,
}

/// A single data point within a historical rate series.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HistoricalRatePoint {
    /// Date of this data point (YYYY-MM-DD).
    pub date: Option<String>,
    /// Exchange rate on this date.
    pub rate: Option<f64>,
    /// Milliseconds since the Unix epoch.
    pub timestamp: Option<i64>,
}

/// One row of an official published table.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OfficialRateEntry {
    /// Base currency code.
    pub base: String,
    /// Quote currency code.
    pub quote: String,
    /// Rate type as the institution labels it, e.g. `reference`, `buy`, `sell`, `mid`.
    #[serde(rename = "type")]
    pub rate_type: String,
    /// The published value.
    pub value: f64,
}

/// Attribution block on official-rate responses.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OfficialRateAttribution {
    /// Attribution text.
    pub source: Option<String>,
    /// Page to link to.
    pub url: Option<String>,
    /// Terms of use.
    pub terms: Option<String>,
}

/// Returned by [`AllRatesToday::official_rates`]. With a pair, `source`, `target`, `rate` and
/// `rate_type` are set; without one, `rates` holds the full table.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OfficialRatesResponse {
    /// Source code, e.g. `ecb`.
    pub bank: String,
    /// Publication date of the table (YYYY-MM-DD). Cite this, not today's date.
    pub rate_date: String,
    /// Full published table (no pair requested).
    pub rates: Option<Vec<OfficialRateEntry>>,
    /// Base currency of the requested pair.
    pub source: Option<String>,
    /// Quote currency of the requested pair.
    pub target: Option<String>,
    /// Rate for the requested pair.
    pub rate: Option<f64>,
    /// Rate type for the requested pair.
    pub rate_type: Option<String>,
    /// True when the pair was cross-computed inside the bank's own table.
    pub derived: Option<bool>,
    /// `published` or `inverse`.
    pub method: Option<String>,
    /// Days since publication.
    pub age_days: Option<i64>,
    /// True when the institution is behind its own publication schedule.
    pub stale: Option<bool>,
    /// Publication cadence, e.g. `daily`.
    pub cadence: Option<String>,
    /// Attribution to display.
    pub attribution: Option<OfficialRateAttribution>,
    /// Note about what needs an API key.
    pub free_key: Option<String>,
}

/// One entry returned by [`AllRatesToday::official_sources`].
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OfficialSource {
    /// Pass this as `bank` to [`AllRatesToday::official_rates`].
    pub code: String,
    /// Institution name.
    pub name: String,
    /// Latest publication date, or `None` if never ingested.
    pub latest: Option<String>,
    /// Days since the latest publication.
    pub age_days: Option<i64>,
    /// True when behind schedule.
    pub stale: Option<bool>,
    /// Publication cadence.
    pub cadence: Option<String>,
}

/// Returned by [`AllRatesToday::official_sources`].
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OfficialSourcesResponse {
    /// Every covered source.
    pub sources: Vec<OfficialSource>,
    /// Number of sources behind schedule.
    pub stale_count: Option<u32>,
    /// When the list was checked.
    pub checked_at: Option<String>,
    /// Documentation URL.
    pub docs: Option<String>,
    /// Note about what needs an API key.
    pub free_key: Option<String>,
}

#[derive(Deserialize)]
struct RawHistory {
    source: Option<String>,
    target: Option<String>,
    period: Option<String>,
    #[serde(default)]
    data: Vec<HistoricalRatePoint>,
}

#[derive(Deserialize)]
struct RawConvertSide {
    amount: f64,
}

#[derive(Deserialize)]
struct RawConvert {
    to: RawConvertSide,
    rate: f64,
}

#[derive(Deserialize)]
struct RawSymbols {
    #[serde(default)]
    currencies: Vec<CurrencySymbol>,
    count: Option<u32>,
}

// ---------------------------------------------------------------------------
// Date helpers (civil calendar, no extra dependency)
// ---------------------------------------------------------------------------

fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn parse_date(s: &str) -> Option<i64> {
    let b = s.as_bytes();
    if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
        return None;
    }
    let y: i64 = s[0..4].parse().ok()?;
    let m: i64 = s[5..7].parse().ok()?;
    let d: i64 = s[8..10].parse().ok()?;
    if !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return None;
    }
    Some(days_from_civil(y, m, d))
}

fn day_of(s: &str) -> &str {
    s.get(..10).unwrap_or(s)
}

// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------

/// The main client for interacting with the AllRatesToday API.
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
    /// Fixed "today" (days since epoch) for tests.
    today: Option<i64>,
}

impl AllRatesToday {
    /// The default base URL for the AllRatesToday API.
    const DEFAULT_BASE_URL: &'static str = "https://allratestoday.com";

    /// Create a new client with the given API key (`art_live_...`).
    /// Get one free at <https://allratestoday.com/register>.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self::with_base_url(api_key, Self::DEFAULT_BASE_URL)
    }

    /// Create a client without an API key, for the keyless methods
    /// ([`symbols`](Self::symbols), [`official_rates`](Self::official_rates),
    /// [`official_sources`](Self::official_sources)).
    pub fn keyless() -> Self {
        Self::new("")
    }

    /// Create a new client with a custom base URL (useful for testing).
    pub fn with_base_url(api_key: impl Into<String>, base_url: impl Into<String>) -> Self {
        let base_url: String = base_url.into();
        Self {
            api_key: api_key.into(),
            base_url: base_url.trim_end_matches('/').to_string(),
            client: reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_else(|_| reqwest::blocking::Client::new()),
            today: None,
        }
    }

    // -- internal helpers ---------------------------------------------------

    fn get<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, String)],
    ) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let mut req = self.client.get(&url).header("Accept", "application/json");
        if !query.is_empty() {
            req = req.query(query);
        }
        if !self.api_key.is_empty() {
            req = req.header("Authorization", format!("Bearer {}", self.api_key));
        }
        let response = req.send()?;
        let status = response.status();
        let body = response.text()?;
        if !status.is_success() {
            return Err(AllRatesTodayError::ApiError {
                status: status.as_u16(),
                message: body.trim().to_string(),
            });
        }
        Ok(serde_json::from_str(&body)?)
    }

    fn require_key(&self) -> Result<()> {
        if self.api_key.is_empty() {
            return invalid(
                "API key is required. Register for free at https://allratestoday.com/register",
            );
        }
        Ok(())
    }

    fn rates(&self, source: &str, targets: &[&str]) -> Result<Vec<Rate>> {
        self.require_key()?;
        let mut q = vec![("source", source.to_string())];
        if !targets.is_empty() {
            q.push(("target", targets.join(",")));
        }
        self.get("/api/v1/rates", &q)
    }

    fn today(&self) -> i64 {
        self.today.unwrap_or_else(|| {
            let secs = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);
            secs.div_euclid(86_400)
        })
    }

    /// Smallest history window ending today that reaches `start`.
    fn window_for(&self, start: &str) -> Result<&'static str> {
        let Some(d) = parse_date(start) else {
            return invalid(format!("Invalid date: {start}. Use YYYY-MM-DD."));
        };
        let days = self.today() - d;
        match days {
            n if n < 0 => invalid(format!("Date {start} is in the future.")),
            0..=6 => Ok("7d"),
            7..=29 => Ok("30d"),
            30..=364 => Ok("1y"),
            _ => invalid(format!(
                "Historical market rates cover the last 365 days; {start} is older."
            )),
        }
    }

    fn require_symbols<'a>(symbols: Option<&'a [&'a str]>, method: &str) -> Result<&'a [&'a str]> {
        match symbols {
            Some(s) if !s.is_empty() => Ok(s),
            _ => invalid(format!("{method}() needs symbols, e.g. Some(&[\"EUR\"]).")),
        }
    }

    fn history_points(&self, source: &str, target: &str, period: &str) -> Result<RawHistory> {
        self.require_key()?;
        let mut raw: RawHistory = self.get(
            "/api/historical-rates",
            &[
                ("source", source.to_string()),
                ("target", target.to_string()),
                ("period", period.to_string()),
            ],
        )?;
        raw.data.sort_by_key(|p| p.timestamp.unwrap_or(0));
        Ok(raw)
    }

    /// Rate on `date`, or the closest earlier point (the 1y window is weekly).
    fn rate_on(&self, source: &str, target: &str, date: &str) -> Result<Option<f64>> {
        let period = self.window_for(date)?;
        let mut rate = None;
        for p in self.history_points(source, target, period)?.data {
            if day_of(p.date.as_deref().unwrap_or("")) > date {
                break;
            }
            if p.rate.is_some() {
                rate = p.rate;
            }
        }
        Ok(rate)
    }

    // -- public API: market rates (API key required) ------------------------

    /// Latest mid-market rates for `base`. Pass `None` for every available currency.
    ///
    /// ```no_run
    /// # use allratestoday::AllRatesToday;
    /// let client = AllRatesToday::new("art_live_your_api_key");
    /// let rates = client.latest("USD", Some(&["EUR", "GBP"])).unwrap();
    /// println!("{:?} {:?}", rates.date, rates.rates);
    /// ```
    pub fn latest(&self, base: &str, symbols: Option<&[&str]>) -> Result<LatestRatesResponse> {
        let list = self.rates(base, symbols.unwrap_or(&[]))?;
        let mut rates = HashMap::with_capacity(list.len());
        let mut date = None;
        for r in list {
            if r.time.is_some() {
                date = r.time.clone();
            }
            rates.insert(r.target, r.rate);
        }
        Ok(LatestRatesResponse {
            success: Some(true),
            base: Some(base.to_string()),
            date,
            rates: Some(rates),
        })
    }

    /// Raw rate list from `GET /api/v1/rates`, one [`Rate`] per target with its observation
    /// time. Pass an empty slice for every available currency.
    pub fn get_rates(&self, source: &str, targets: &[&str]) -> Result<Vec<Rate>> {
        self.rates(source, targets)
    }

    /// Current rate for a single currency pair.
    ///
    /// ```no_run
    /// # use allratestoday::AllRatesToday;
    /// let client = AllRatesToday::new("art_live_your_api_key");
    /// let rate = client.get_rate("USD", "EUR").unwrap();
    /// println!("USD/EUR: {:?} at {:?}", rate.rate, rate.date);
    /// ```
    pub fn get_rate(&self, from: &str, to: &str) -> Result<RateResponse> {
        let r = self.rates(from, &[to])?.into_iter().next().ok_or_else(|| {
            AllRatesTodayError::ParseError(format!("No rate returned for {from}/{to}"))
        })?;
        Ok(RateResponse {
            success: Some(true),
            from: Some(r.source),
            to: Some(r.target),
            rate: Some(r.rate),
            date: r.time,
        })
    }

    /// Convert an amount at the current rate.
    ///
    /// ```no_run
    /// # use allratestoday::AllRatesToday;
    /// let client = AllRatesToday::new("art_live_your_api_key");
    /// let result = client.convert("USD", "EUR", 100.0).unwrap();
    /// println!("Converted: {:?}", result.result);
    /// ```
    pub fn convert(&self, from: &str, to: &str, amount: f64) -> Result<ConvertResponse> {
        self.require_key()?;
        let raw: RawConvert = self.get(
            "/api/v1/rates",
            &[
                ("source", from.to_string()),
                ("target", to.to_string()),
                ("amount", amount.to_string()),
            ],
        )?;
        Ok(ConvertResponse {
            success: Some(true),
            from: Some(from.to_string()),
            to: Some(to.to_string()),
            amount: Some(amount),
            result: Some(raw.to.amount),
            rate: Some(raw.rate),
            date: None,
        })
    }

    /// Convert an amount at the rate on a past date (`YYYY-MM-DD`, last 365 days). Uses the
    /// closest earlier point when the date has none (the 1-year window is weekly). Paid plans
    /// only: free and trial keys get a 403 `historical_requires_paid_plan` error.
    pub fn convert_on_date(
        &self,
        from: &str,
        to: &str,
        amount: f64,
        date: &str,
    ) -> Result<ConvertResponse> {
        let Some(rate) = self.rate_on(from, to, date)? else {
            return Err(AllRatesTodayError::ParseError(format!(
                "No {from}/{to} rate available on or before {date}"
            )));
        };
        let result = (amount * rate * 1e6).round() / 1e6;
        Ok(ConvertResponse {
            success: Some(true),
            from: Some(from.to_string()),
            to: Some(to.to_string()),
            amount: Some(amount),
            result: Some(result),
            rate: Some(rate),
            date: Some(date.to_string()),
        })
    }

    /// Rates on a past date (`YYYY-MM-DD`, last 365 days). `symbols` is required; each symbol
    /// costs one request because the history endpoint takes one target per call. Paid plans
    /// only: free and trial keys get a 403 `historical_requires_paid_plan` error.
    ///
    /// ```no_run
    /// # use allratestoday::AllRatesToday;
    /// let client = AllRatesToday::new("art_live_your_api_key");
    /// let rates = client.for_date("2026-10-01", "USD", Some(&["EUR"])).unwrap();
    /// println!("Historical: {:?}", rates.rates);
    /// ```
    pub fn for_date(
        &self,
        date: &str,
        base: &str,
        symbols: Option<&[&str]>,
    ) -> Result<HistoricalRatesResponse> {
        let symbols = Self::require_symbols(symbols, "for_date")?;
        self.window_for(date)?;
        let mut rates = HashMap::new();
        for target in symbols {
            if let Some(rate) = self.rate_on(base, target, date)? {
                rates.insert(target.to_string(), rate);
            }
        }
        Ok(HistoricalRatesResponse {
            success: Some(true),
            base: Some(base.to_string()),
            date: Some(date.to_string()),
            rates: Some(rates),
        })
    }

    /// Daily rates between two dates (`YYYY-MM-DD`, start within the last 365 days), keyed by
    /// date then currency. `symbols` is required; one request per symbol. Paid plans only:
    /// free and trial keys get a 403 `historical_requires_paid_plan` error.
    ///
    /// ```no_run
    /// # use allratestoday::AllRatesToday;
    /// let client = AllRatesToday::new("art_live_your_api_key");
    /// let series = client
    ///     .time_series("2026-10-01", "2026-10-15", "USD", Some(&["EUR", "GBP"]))
    ///     .unwrap();
    /// println!("Time series: {:?}", series.rates);
    /// ```
    pub fn time_series(
        &self,
        start_date: &str,
        end_date: &str,
        base: &str,
        symbols: Option<&[&str]>,
    ) -> Result<TimeSeriesResponse> {
        let symbols = Self::require_symbols(symbols, "time_series")?;
        let period = self.window_for(start_date)?;
        let mut rates: HashMap<String, HashMap<String, f64>> = HashMap::new();
        for target in symbols {
            for p in self.history_points(base, target, period)?.data {
                let (Some(date), Some(rate)) = (p.date, p.rate) else {
                    continue;
                };
                let d = day_of(&date);
                if d < start_date || d > end_date {
                    continue;
                }
                rates
                    .entry(d.to_string())
                    .or_default()
                    .insert(target.to_string(), rate);
            }
        }
        Ok(TimeSeriesResponse {
            success: Some(true),
            base: Some(base.to_string()),
            start_date: Some(start_date.to_string()),
            end_date: Some(end_date.to_string()),
            rates: Some(rates),
        })
    }

    /// A pair's history over a preset period: `"1d"`, `"7d"`, `"30d"` or `"1y"` (daily points;
    /// weekly for `"1y"`), oldest first. Paid plans only: free and trial keys get a 403
    /// `historical_requires_paid_plan` error.
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
        let raw = self.history_points(source, target, period)?;
        Ok(HistoricalRateSeriesResponse {
            success: Some(true),
            source: raw.source.or_else(|| Some(source.to_string())),
            target: raw.target.or_else(|| Some(target.to_string())),
            period: raw.period.or_else(|| Some(period.to_string())),
            rates: Some(raw.data),
        })
    }

    // -- public API: keyless -------------------------------------------------

    /// Every supported currency. No API key needed.
    ///
    /// ```no_run
    /// # use allratestoday::AllRatesToday;
    /// let symbols = AllRatesToday::keyless().symbols().unwrap();
    /// println!("{:?} currencies", symbols.count);
    /// ```
    pub fn symbols(&self) -> Result<SymbolsResponse> {
        let raw: RawSymbols = self.get("/api/v1/symbols", &[])?;
        let mut map = HashMap::with_capacity(raw.currencies.len());
        for c in &raw.currencies {
            if let Some(code) = &c.code {
                map.insert(code.clone(), serde_json::to_value(c)?);
            }
        }
        Ok(SymbolsResponse {
            success: Some(true),
            symbols: Some(map),
            count: raw.count.or(Some(raw.currencies.len() as u32)),
            currencies: Some(raw.currencies),
        })
    }

    /// Latest official table published by a central bank or tax authority (e.g. `"ecb"`,
    /// `"fed"`, `"hmrc"`). Pass `source` and `target` for one pair, or `None` for both to get
    /// the full table. No API key needed.
    ///
    /// ```no_run
    /// # use allratestoday::AllRatesToday;
    /// let ecb = AllRatesToday::keyless().official_rates("ecb", Some("EUR"), Some("USD")).unwrap();
    /// println!("ECB {}: 1 EUR = {:?} USD", ecb.rate_date, ecb.rate);
    /// ```
    pub fn official_rates(
        &self,
        bank: &str,
        source: Option<&str>,
        target: Option<&str>,
    ) -> Result<OfficialRatesResponse> {
        let code = bank.trim().to_lowercase();
        let ok = (2..=20).contains(&code.len())
            && code
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-');
        if !ok {
            return invalid(format!(
                "Invalid source code: {bank}. Call official_sources() for the list."
            ));
        }
        let q = match (source, target) {
            (Some(s), Some(t)) => vec![("source", s.to_uppercase()), ("target", t.to_uppercase())],
            (None, None) => vec![],
            _ => return invalid("Provide both source and target, or neither."),
        };
        self.get(&format!("/api/open/central-bank/{code}"), &q)
    }

    /// Every covered official source with its code, name, latest publication date and whether
    /// it is behind schedule. No API key needed.
    pub fn official_sources(&self) -> Result<OfficialSourcesResponse> {
        self.get("/api/open/central-banks", &[])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufRead, BufReader, Write};
    use std::net::TcpListener;
    use std::sync::{Arc, Mutex};

    // Response bodies captured from the live API (trimmed).
    const RATES: &str = r#"[{"rate":0.888455,"source":"USD","target":"EUR","time":"2026-10-16T12:04:01+0000"},{"rate":0.749738,"source":"USD","target":"GBP","time":"2026-10-16T12:04:01+0000"}]"#;
    const RATE: &str =
        r#"[{"rate":0.888455,"source":"USD","target":"EUR","time":"2026-10-16T12:04:01+0000"}]"#;
    const CONVERT: &str = r#"{"from":{"currency":"USD","amount":100},"to":{"currency":"EUR","amount":88.8455},"rate":0.888455,"source":"mid-market"}"#;
    const HIST_EUR: &str = r#"{"source":"USD","target":"EUR","data":[{"date":"2026-10-14","rate":0.861,"timestamp":1791936000000},{"date":"2026-10-12","rate":0.859,"timestamp":1791763200000},{"date":"2026-10-15","rate":0.862,"timestamp":1792022400000},{"date":"2026-10-16","rate":0.888455,"timestamp":1792108800000}],"source_api":"mid-market","period":"7d"}"#;
    const HIST_GBP: &str = r#"{"source":"USD","target":"GBP","data":[{"date":"2026-10-12","rate":0.741,"timestamp":1791763200000},{"date":"2026-10-15","rate":0.745,"timestamp":1792022400000}],"source_api":"mid-market","period":"7d"}"#;
    const SYMBOLS: &str = r#"{"currencies":[{"code":"USD","name":"US Dollar","symbol":"$"},{"code":"EUR","name":"Euro","symbol":"€"}],"count":2}"#;
    const OFFICIAL: &str = r#"{"bank":"ecb","rate_date":"2026-10-15","source":"EUR","target":"USD","rate":1.1208,"rate_type":"reference","derived":false,"method":"published","age_days":1,"stale":false,"cadence":"daily","attribution":{"source":"Official rates published by ECB, served by AllRatesToday","url":"https://allratestoday.com/central-bank-rates-api/ecb/","terms":"Free to use with visible attribution linking to allratestoday.com"},"free_key":"..."}"#;
    const OFFICIAL_TABLE: &str = r#"{"bank":"ecb","rate_date":"2026-10-15","age_days":1,"stale":false,"cadence":"daily","rates":[{"base":"EUR","quote":"AUD","type":"reference","value":1.6218},{"base":"EUR","quote":"BRL","type":"reference","value":5.6679}]}"#;
    const SOURCES: &str = r#"{"sources":[{"code":"amcm","name":"Monetary Authority of Macao","latest":"2026-10-16","age_days":0,"stale":false,"cadence":"daily"}],"stale_count":3,"checked_at":"2026-10-16T12:04:27.421Z","docs":"https://allratestoday.com/central-bank-rates-api/","free_key":"..."}"#;

    /// A recorded request: decoded path+query and Authorization header.
    #[derive(Debug, Clone)]
    struct Seen {
        target: String,
        auth: Option<String>,
    }

    fn decode(s: &str) -> String {
        s.replace("%2C", ",").replace("%2c", ",")
    }

    /// Minimal HTTP/1.1 mock: `route` maps a decoded "path?query" to (status, body).
    fn mock(route: fn(&str) -> (u16, &'static str)) -> (String, Arc<Mutex<Vec<Seen>>>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = format!("http://{}", listener.local_addr().unwrap());
        let seen = Arc::new(Mutex::new(Vec::new()));
        let log = seen.clone();
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { continue };
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                let target = decode(line.split_whitespace().nth(1).unwrap_or(""));
                let mut auth = None;
                loop {
                    let mut h = String::new();
                    if reader.read_line(&mut h).unwrap() == 0 || h == "\r\n" {
                        break;
                    }
                    if let Some((k, v)) = h.split_once(':') {
                        if k.eq_ignore_ascii_case("authorization") {
                            auth = Some(v.trim().to_string());
                        }
                    }
                }
                let (status, body) = route(&target);
                log.lock().unwrap().push(Seen { target, auth });
                let resp = format!(
                    "HTTP/1.1 {status} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = stream.write_all(resp.as_bytes());
            }
        });
        (addr, seen)
    }

    fn route(t: &str) -> (u16, &'static str) {
        match t {
            "/api/v1/rates?source=USD&target=EUR,GBP" => (200, RATES),
            "/api/v1/rates?source=EUR" => (200, RATES),
            "/api/v1/rates?source=USD&target=EUR" => (200, RATE),
            "/api/v1/rates?source=USD&target=EUR&amount=100" => (200, CONVERT),
            "/api/historical-rates?source=USD&target=EUR&period=7d" => (200, HIST_EUR),
            "/api/historical-rates?source=USD&target=GBP&period=7d" => (200, HIST_GBP),
            "/api/v1/symbols" => (200, SYMBOLS),
            "/api/open/central-bank/ecb?source=EUR&target=USD" => (200, OFFICIAL),
            "/api/open/central-bank/ecb" => (200, OFFICIAL_TABLE),
            "/api/open/central-banks" => (200, SOURCES),
            _ => (404, r#"{"error":"Not found"}"#),
        }
    }

    /// Client against the mock with "today" fixed at 2026-10-16.
    fn client(key: &str) -> (AllRatesToday, Arc<Mutex<Vec<Seen>>>) {
        let (addr, seen) = mock(route);
        let mut c = AllRatesToday::with_base_url(key, addr);
        c.today = parse_date("2026-10-16");
        (c, seen)
    }

    const KEY: &str = "art_live_test_key";

    #[test]
    fn client_defaults() {
        let c = AllRatesToday::new("art_live_test_key_123");
        assert_eq!(c.api_key, "art_live_test_key_123");
        assert_eq!(c.base_url, "https://allratestoday.com");
        let c = AllRatesToday::with_base_url("k", "https://custom.example.com/");
        assert_eq!(c.base_url, "https://custom.example.com");
    }

    #[test]
    fn latest() {
        let (c, seen) = client(KEY);
        let r = c.latest("USD", Some(&["EUR", "GBP"])).unwrap();
        assert_eq!(r.base.as_deref(), Some("USD"));
        assert_eq!(r.date.as_deref(), Some("2026-10-16T12:04:01+0000"));
        let rates = r.rates.unwrap();
        assert_eq!(rates["EUR"], 0.888455);
        assert_eq!(rates["GBP"], 0.749738);
        assert_eq!(
            seen.lock().unwrap()[0].auth.as_deref(),
            Some("Bearer art_live_test_key")
        );
    }

    #[test]
    fn latest_all_symbols_omits_target() {
        let (c, seen) = client(KEY);
        c.latest("EUR", None).unwrap();
        assert_eq!(seen.lock().unwrap()[0].target, "/api/v1/rates?source=EUR");
    }

    #[test]
    fn get_rates_and_get_rate() {
        let (c, _) = client(KEY);
        let list = c.get_rates("USD", &["EUR", "GBP"]).unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[1].target, "GBP");
        let r = c.get_rate("USD", "EUR").unwrap();
        assert_eq!(r.from.as_deref(), Some("USD"));
        assert_eq!(r.to.as_deref(), Some("EUR"));
        assert_eq!(r.rate, Some(0.888455));
        assert_eq!(r.date.as_deref(), Some("2026-10-16T12:04:01+0000"));
    }

    #[test]
    fn convert() {
        let (c, _) = client(KEY);
        let r = c.convert("USD", "EUR", 100.0).unwrap();
        assert_eq!(r.amount, Some(100.0));
        assert_eq!(r.result, Some(88.8455));
        assert_eq!(r.rate, Some(0.888455));
        assert_eq!(r.date, None);
    }

    #[test]
    fn convert_on_date_uses_closest_earlier_point() {
        let (c, _) = client(KEY);
        let r = c
            .convert_on_date("USD", "EUR", 1000.0, "2026-10-13")
            .unwrap();
        assert_eq!(r.rate, Some(0.859));
        assert_eq!(r.result, Some(859.0));
        assert_eq!(r.date.as_deref(), Some("2026-10-13"));
    }

    #[test]
    fn for_date_one_call_per_symbol() {
        let (c, seen) = client(KEY);
        let r = c
            .for_date("2026-10-14", "USD", Some(&["EUR", "GBP"]))
            .unwrap();
        let rates = r.rates.unwrap();
        assert_eq!(rates["EUR"], 0.861);
        assert_eq!(rates["GBP"], 0.741);
        assert_eq!(seen.lock().unwrap().len(), 2);
    }

    #[test]
    fn for_date_validation() {
        let (c, seen) = client(KEY);
        assert!(matches!(
            c.for_date("2026-10-01", "USD", None),
            Err(AllRatesTodayError::InvalidArgument(_))
        ));
        for d in ["2025-01-15", "2026-10-17", "15/01/2026"] {
            assert!(
                matches!(
                    c.for_date(d, "USD", Some(&["EUR"])),
                    Err(AllRatesTodayError::InvalidArgument(_))
                ),
                "{d}"
            );
        }
        assert!(seen.lock().unwrap().is_empty());
    }

    #[test]
    fn window_for() {
        let mut c = AllRatesToday::new("k");
        c.today = parse_date("2026-10-16");
        for (d, want) in [
            ("2026-10-16", "7d"),
            ("2026-10-10", "7d"),
            ("2026-10-09", "30d"),
            ("2026-09-17", "30d"),
            ("2026-09-16", "1y"),
            ("2025-10-17", "1y"),
        ] {
            assert_eq!(c.window_for(d).unwrap(), want, "{d}");
        }
        assert!(c.window_for("2025-10-16").is_err());
    }

    #[test]
    fn time_series() {
        let (c, _) = client(KEY);
        let r = c
            .time_series("2026-10-12", "2026-10-15", "USD", Some(&["EUR", "GBP"]))
            .unwrap();
        let rates = r.rates.unwrap();
        assert_eq!(rates.len(), 3, "{rates:?}");
        assert_eq!(rates["2026-10-15"]["EUR"], 0.862);
        assert_eq!(rates["2026-10-15"]["GBP"], 0.745);
        assert!(!rates.contains_key("2026-10-16"));
    }

    #[test]
    fn get_historical_rates_sorted() {
        let (c, _) = client(KEY);
        let r = c.get_historical_rates("USD", "EUR", "7d").unwrap();
        assert_eq!(r.period.as_deref(), Some("7d"));
        let pts = r.rates.unwrap();
        assert_eq!(pts.len(), 4);
        assert_eq!(pts[0].date.as_deref(), Some("2026-10-12"));
        assert_eq!(pts[3].timestamp, Some(1792108800000));
    }

    #[test]
    fn symbols_keyless() {
        let (c, seen) = client("");
        let r = c.symbols().unwrap();
        assert_eq!(r.count, Some(2));
        assert_eq!(r.currencies.unwrap()[1].symbol.as_deref(), Some("€"));
        assert_eq!(r.symbols.unwrap()["EUR"]["name"], "Euro");
        assert!(seen.lock().unwrap()[0].auth.is_none());
    }

    #[test]
    fn official_rates_keyless() {
        let (c, seen) = client("");
        let r = c.official_rates("ECB", Some("eur"), Some("usd")).unwrap();
        assert_eq!(r.rate_date, "2026-10-15");
        assert_eq!(r.rate, Some(1.1208));
        assert_eq!(r.rate_type.as_deref(), Some("reference"));
        let t = c.official_rates("ecb", None, None).unwrap();
        let rows = t.rates.unwrap();
        assert_eq!(rows[0].quote, "AUD");
        assert_eq!(rows[0].rate_type, "reference");
        let s = c.official_sources().unwrap();
        assert_eq!(s.sources[0].code, "amcm");
        assert_eq!(s.stale_count, Some(3));
        assert!(seen.lock().unwrap().iter().all(|s| s.auth.is_none()));
        assert!(c.official_rates("ecb", Some("EUR"), None).is_err());
        assert!(c.official_rates("../x", None, None).is_err());
    }

    #[test]
    fn missing_key() {
        let c = AllRatesToday::with_base_url("", "http://127.0.0.1:1");
        assert!(matches!(
            c.latest("USD", None),
            Err(AllRatesTodayError::InvalidArgument(_))
        ));
    }

    #[test]
    fn api_error() {
        let (c, _) = client(KEY);
        match c.get_rate("USD", "XXX") {
            Err(AllRatesTodayError::ApiError { status, message }) => {
                assert_eq!(status, 404);
                assert_eq!(message, r#"{"error":"Not found"}"#);
            }
            other => panic!("expected ApiError, got {other:?}"),
        }
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

    #[test]
    fn days_from_civil_known_values() {
        assert_eq!(parse_date("1970-01-01"), Some(0));
        assert_eq!(parse_date("2026-10-16"), Some(1792108800000 / 86_400_000));
    }

    /// Live smoke test; runs only when ALLRATES_API_KEY is set (4 keyed requests).
    #[test]
    fn live_smoke() {
        let Ok(key) = std::env::var("ALLRATES_API_KEY") else {
            return;
        };
        let c = AllRatesToday::new(key);
        let latest = c.latest("USD", Some(&["EUR", "GBP"])).unwrap();
        assert!(latest.rates.as_ref().unwrap()["EUR"] > 0.0);
        let conv = c.convert("USD", "EUR", 100.0).unwrap();
        assert!(conv.result.unwrap() > 0.0);
        let today = c.today();
        let past = days_to_string(today - 3);
        // History is paid-plan only; on a free or trial key a 403
        // historical_requires_paid_plan counts as a pass.
        let paid_only = |e: &AllRatesTodayError| matches!(e, AllRatesTodayError::ApiError { status: 403, message } if message.contains("historical_requires_paid_plan"));
        let hist_eur = match c.for_date(&past, "USD", Some(&["EUR"])) {
            Ok(h) => {
                let v = h.rates.unwrap()["EUR"];
                assert!(v > 0.0);
                Some(v)
            }
            Err(e) => {
                assert!(paid_only(&e), "{e}");
                None
            }
        };
        let points = match c.get_historical_rates("USD", "EUR", "7d") {
            Ok(s) => {
                let n = s.rates.unwrap().len();
                assert!(n > 0);
                n
            }
            Err(e) => {
                assert!(paid_only(&e), "{e}");
                0
            }
        };
        let sym = AllRatesToday::keyless().symbols().unwrap();
        assert!(sym.count.unwrap() >= 150);
        let ecb = AllRatesToday::keyless()
            .official_rates("ecb", Some("EUR"), Some("USD"))
            .unwrap();
        assert!(ecb.rate.unwrap() > 0.0);
        eprintln!(
            "LIVE: USD/EUR {:?} ({:?}); 100 USD = {:?} EUR; {past} EUR {:?}; {} points; {:?} symbols; ECB {} {:?}",
            latest.rates.unwrap()["EUR"], latest.date, conv.result, hist_eur,
            points, sym.count, ecb.rate_date, ecb.rate
        );
    }

    fn days_to_string(z: i64) -> String {
        // Inverse of days_from_civil (Howard Hinnant's civil_from_days).
        let z = z + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z - era * 146_097;
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = doy - (153 * mp + 2) / 5 + 1;
        let m = if mp < 10 { mp + 3 } else { mp - 9 };
        let y = yoe + era * 400 + if m <= 2 { 1 } else { 0 };
        format!("{y:04}-{m:02}-{d:02}")
    }
}
