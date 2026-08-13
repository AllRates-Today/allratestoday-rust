# AllRatesToday Rust SDK — allratestoday

The official Rust client for the [AllRatesToday](https://allratestoday.com) currency API. It is a small, blocking HTTP wrapper for Rust services and CLI tools that need real-time, historical or time-series exchange rates without hand-rolling request and response types.

[![Powered by AllRatesToday](https://img.shields.io/badge/Powered%20by-AllRatesToday-orange.svg)](https://allratestoday.com)

[![Crates.io](https://img.shields.io/crates/v/allratestoday.svg)](https://crates.io/crates/allratestoday)
[![Docs.rs](https://docs.rs/allratestoday/badge.svg)](https://docs.rs/allratestoday)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust 2021](https://img.shields.io/badge/rust-2021%20edition-orange.svg)](https://www.rust-lang.org/)
[![Powered by AllRatesToday](https://img.shields.io/badge/Powered%20by-AllRatesToday-orange.svg)](https://allratestoday.com)

## 🚀 Features

- ⚡ **Real-time rates** — latest mid-market rates for any base currency
- 🌍 **160+ currencies** — the full list is available from the API via `symbols()`
- 📅 **Historical data** — rates for a single past date, an arbitrary date range, or a preset period (`1d`, `7d`, `30d`, `1y`)
- 💱 **Conversion** — convert an amount between two currencies in one call
- 🧩 **Typed responses** — every endpoint deserialises into a documented `struct` via `serde`
- 🛡️ **Typed errors** — one `AllRatesTodayError` enum separating transport, API and parse failures
- 🔧 **Overridable base URL** — point the client at a test server with `with_base_url`
- 📡 **Data source** — institutional interbank market data

## 🔑 Get your API key

Create a free key at [allratestoday.com/register](https://allratestoday.com/register). Keys look like `art_live_...`, and the SDK sends yours as a Bearer token in the `Authorization` header on every request.

## 📦 Installation

```bash
cargo add allratestoday
```

Or add it to `Cargo.toml` manually:

```toml
[dependencies]
allratestoday = "1"
```

The crate builds on `reqwest` (blocking, `rustls-tls`), `serde` and `serde_json`.

## 🏁 Quick start

```rust
use allratestoday::AllRatesToday;

fn main() {
    let client = AllRatesToday::new("art_live_your_api_key");

    // Latest rates for a base currency
    let rates = client.latest("USD", Some(&["EUR", "GBP"])).unwrap();
    println!("Base: {:?}", rates.base);
    println!("Rates: {:?}", rates.rates);

    // Convert an amount
    let result = client.convert("USD", "EUR", 100.0).unwrap();
    println!("100 USD = {:?} EUR", result.result);
}
```

Prefer to keep the key out of your source? The bundled example reads it from the environment:

```bash
export ALLRATESTODAY_API_KEY="art_live_your_api_key"
cargo run --example basic
```

## 📚 API reference

Construct a client with `AllRatesToday::new(api_key)`, or `AllRatesToday::with_base_url(api_key, base_url)` to target a different host. Every method is blocking and returns `Result<T, AllRatesTodayError>`.

| Method | Endpoint | Returns |
|--------|----------|---------|
| `latest(base, symbols)` | `GET /v1/latest` | `LatestRatesResponse` |
| `convert(from, to, amount)` | `GET /v1/convert` | `ConvertResponse` |
| `for_date(date, base, symbols)` | `GET /v1/historical` | `HistoricalRatesResponse` |
| `time_series(start_date, end_date, base, symbols)` | `GET /v1/timeseries` | `TimeSeriesResponse` |
| `symbols()` | `GET /v1/symbols` | `SymbolsResponse` |
| `get_rate(from, to)` | `GET /v1/rate` | `RateResponse` |
| `get_historical_rates(source, target, period)` | `GET /v1/historical-rates` | `HistoricalRateSeriesResponse` |

`symbols` parameters take `Option<&[&str]>`; pass `None` for every available currency.

### `latest(base, symbols)`

Most recent rates for a base currency.

```rust
// All currencies
let rates = client.latest("USD", None)?;

// Specific currencies only
let rates = client.latest("USD", Some(&["EUR", "GBP", "JPY"]))?;
println!("Date: {:?}", rates.date);
println!("Rates: {:?}", rates.rates);
```

### `convert(from, to, amount)`

Convert an amount between two currencies.

```rust
let result = client.convert("USD", "EUR", 250.0)?;
println!("Rate: {:?}", result.rate);
println!("Result: {:?}", result.result);
```

### `for_date(date, base, symbols)`

Rates as of a specific historical date (`YYYY-MM-DD`).

```rust
let rates = client.for_date("2025-01-15", "USD", Some(&["EUR", "GBP"]))?;
println!("Date: {:?}", rates.date);
println!("Rates: {:?}", rates.rates);
```

### `time_series(start_date, end_date, base, symbols)`

Rates across a date range, keyed by date.

```rust
let series = client.time_series(
    "2025-01-01",
    "2025-01-31",
    "USD",
    Some(&["EUR", "GBP"]),
)?;
for (date, day_rates) in series.rates.unwrap_or_default() {
    println!("{date}: {day_rates:?}");
}
```

### `symbols()`

Every supported currency code with its details.

```rust
let symbols = client.symbols()?;
for (code, info) in symbols.symbols.unwrap_or_default() {
    println!("{code}: {info}");
}
```

### `get_rate(from, to)`

A single currency pair.

```rust
let rate = client.get_rate("USD", "EUR")?;
println!("USD/EUR: {:?}", rate.rate);
```

### `get_historical_rates(source, target, period)`

A pair's history over a preset period — `"1d"`, `"7d"`, `"30d"` or `"1y"`. Returns a `Vec<HistoricalRatePoint>` of `date` / `rate` pairs.

```rust
let history = client.get_historical_rates("USD", "EUR", "30d")?;
for point in history.rates.unwrap_or_default() {
    println!("{}: {:?}", point.date.unwrap_or_default(), point.rate);
}
```

### `with_base_url(api_key, base_url)`

Override the default host (`https://allratestoday.com`) for testing or a self-hosted deployment.

```rust
let client = AllRatesToday::with_base_url(
    "art_live_your_api_key",
    "https://custom.example.com",
);
```

## 🗺️ Currencies covered

160+ currencies, including 🇺🇸 `USD`, 🇪🇺 `EUR`, 🇬🇧 `GBP` and 🇯🇵 `JPY`. Call `symbols()` for the authoritative, always-current list rather than hard-coding one.

## 🛡️ Error handling

Every method returns `Result<T, AllRatesTodayError>`. The enum has three variants:

| Variant | Description |
|---------|-------------|
| `HttpError(reqwest::Error)` | Network or transport-level failure (timeout, DNS, TLS) |
| `ApiError { status, message }` | Non-2xx HTTP response; `status` is the code, `message` the response body |
| `ParseError(String)` | The response body could not be deserialised |

```rust
use allratestoday::{AllRatesToday, AllRatesTodayError};

let client = AllRatesToday::new("art_live_your_api_key");

match client.latest("USD", None) {
    Ok(rates) => println!("{rates:?}"),
    Err(AllRatesTodayError::HttpError(e)) => eprintln!("Network error: {e}"),
    Err(AllRatesTodayError::ApiError { status, message }) => {
        eprintln!("API error {status}: {message}");
    }
    Err(AllRatesTodayError::ParseError(msg)) => eprintln!("Parse error: {msg}"),
}
```

`AllRatesTodayError` implements `std::error::Error` and `Display`, and converts from `reqwest::Error` and `serde_json::Error`, so `?` works inside functions returning `Box<dyn Error>` or the crate's own `allratestoday::Result<T>` alias.

## 💡 Notes

- **Blocking only.** The client uses `reqwest::blocking`, so calls must not be made from inside an async runtime's worker thread — wrap them in `spawn_blocking` if you are on Tokio.
- **Optional fields.** Response fields are `Option<_>` so a partial or evolving API payload never fails to deserialise. Unwrap or pattern-match rather than assuming a value is present.
- **Tests.** Run the unit tests, which cover client construction and response deserialisation, with `cargo test`.

## 🔗 Links

- **Website:** [allratestoday.com](https://allratestoday.com)
- **API docs:** [allratestoday.com/docs](https://allratestoday.com/docs)
- **Free API key:** [allratestoday.com/register](https://allratestoday.com/register)
- **Crate:** [crates.io/crates/allratestoday](https://crates.io/crates/allratestoday)
- **Reference docs:** [docs.rs/allratestoday](https://docs.rs/allratestoday)
- **Repository:** [github.com/AllRates-Today/allratestoday-rust](https://github.com/AllRates-Today/allratestoday-rust)
- **Status:** [allratestoday.com/status](https://allratestoday.com/status)
- **Support:** [allratestoday.com/contact](https://allratestoday.com/contact)

## 📜 License

MIT — see [LICENSE](LICENSE).
