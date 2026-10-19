# AllRatesToday Rust SDK — allratestoday

The official Rust client for the [AllRatesToday](https://allratestoday.com) currency API. It is a small, blocking HTTP wrapper for Rust services and CLI tools that need real-time, historical or official central-bank exchange rates without hand-rolling request and response types.

[![Powered by AllRatesToday](https://img.shields.io/badge/Powered%20by-AllRatesToday-orange.svg)](https://allratestoday.com)

[![Crates.io](https://img.shields.io/crates/v/allratestoday.svg)](https://crates.io/crates/allratestoday)
[![Docs.rs](https://docs.rs/allratestoday/badge.svg)](https://docs.rs/allratestoday)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust 2021](https://img.shields.io/badge/rust-2021%20edition-orange.svg)](https://www.rust-lang.org/)
[![Powered by AllRatesToday](https://img.shields.io/badge/Powered%20by-AllRatesToday-orange.svg)](https://allratestoday.com)

## 🚀 Features

- ⚡ **Real-time rates** — latest mid-market rates for any base currency
- 🌍 **160+ currencies** — the full list is available keylessly via `symbols()`
- 📅 **Historical data** — a past date, a date range, or a preset period (`1d`, `7d`, `30d`, `1y`) across the last 365 days (paid plans)
- 💱 **Conversion** — convert an amount at the current rate, or at a past date's rate on a paid plan
- 🏛️ **Official rates** — the published tables of 121 central banks + 3 tax authorities, no key needed
- 🧩 **Typed responses** — every method returns a documented `struct` via `serde`
- 🛡️ **Typed errors** — one `AllRatesTodayError` enum separating transport, API, parse and argument failures
- 🔧 **Overridable base URL** — point the client at a test server with `with_base_url`
- 💹 **Mid-market rates** — no retail spread baked in

## 🔑 Get your API key

Create a free key at [allratestoday.com/register](https://allratestoday.com/register): 300 free requests, lifetime (the allowance never resets). Keys look like `art_live_...`, and the SDK sends yours as a Bearer token in the `Authorization` header. `symbols()`, `official_rates()` and `official_sources()` work without a key (use `AllRatesToday::keyless()`); every other method returns `AllRatesTodayError::InvalidArgument` if the client has none.

A free key (and a 100-call agent trial key) covers the latest rates and conversion. History and past-date rates (`convert_on_date`, `for_date`, `time_series`, `get_historical_rates`) need a [paid plan](https://allratestoday.com/pricing); on a free or trial key they return `AllRatesTodayError::ApiError` with status 403 and code `historical_requires_paid_plan`. The keyed API sends no CORS headers, so use it from server-side code, not the browser.

## 📦 Installation

```bash
cargo add allratestoday
```

Or add it to `Cargo.toml` manually:

```toml
[dependencies]
allratestoday = "1.1"
```

The crate builds on `reqwest` (blocking, `rustls-tls`), `serde` and `serde_json`.

> **Upgrading from 1.0.x:** 1.0.x called `/v1/...` paths that the API does not serve, so every call failed with HTTP 404. 1.1.0 keeps the same method names and response types but calls the real endpoints. `for_date` and `time_series` now require at least one symbol and cover the last 365 days. `AllRatesTodayError` gains an `InvalidArgument` variant.

## 🏁 Quick start

```rust
use allratestoday::AllRatesToday;

fn main() {
    let client = AllRatesToday::new("art_live_your_api_key");

    // Latest rates for a base currency
    let rates = client.latest("USD", Some(&["EUR", "GBP"])).unwrap();
    println!("Base: {:?} as of {:?}", rates.base, rates.date);
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

Construct a client with `AllRatesToday::new(api_key)`, `AllRatesToday::keyless()`, or `AllRatesToday::with_base_url(api_key, base_url)` to target a different host. Every method is blocking and returns `Result<T, AllRatesTodayError>`.

| Method | Endpoint | Key | Returns |
|--------|----------|-----|---------|
| `latest(base, symbols)` | `GET /api/v1/rates` | yes | `LatestRatesResponse` |
| `get_rates(source, targets)` | `GET /api/v1/rates` | yes | `Vec<Rate>` |
| `get_rate(from, to)` | `GET /api/v1/rates` | yes | `RateResponse` |
| `convert(from, to, amount)` | `GET /api/v1/rates?amount=` | yes | `ConvertResponse` |
| `convert_on_date(from, to, amount, date)` | `GET /api/historical-rates` | paid plan | `ConvertResponse` |
| `for_date(date, base, symbols)` | `GET /api/historical-rates` (one call per symbol) | paid plan | `HistoricalRatesResponse` |
| `time_series(start_date, end_date, base, symbols)` | `GET /api/historical-rates` (one call per symbol) | paid plan | `TimeSeriesResponse` |
| `get_historical_rates(source, target, period)` | `GET /api/historical-rates` | paid plan | `HistoricalRateSeriesResponse` |
| `symbols()` | `GET /api/v1/symbols` | no | `SymbolsResponse` |
| `official_rates(bank, source, target)` | `GET /api/open/central-bank/{bank}` | no | `OfficialRatesResponse` |
| `official_sources()` | `GET /api/open/central-banks` | no | `OfficialSourcesResponse` |

### `latest(base, symbols)`

Most recent mid-market rates for a base currency. Pass `None` for every available currency. `date` is the observation time, e.g. `2026-10-16T11:31:01+0000`.

```rust
// All currencies
let rates = client.latest("USD", None)?;

// Specific currencies only
let rates = client.latest("USD", Some(&["EUR", "GBP", "JPY"]))?;
println!("Date: {:?}", rates.date);
println!("Rates: {:?}", rates.rates);
```

### `get_rates(source, targets)` and `get_rate(from, to)`

`get_rates` returns the raw list, one `Rate` (`rate`, `source`, `target`, `time`) per target; pass `&[]` for every currency. `get_rate` is the single-pair shortcut.

```rust
for r in client.get_rates("USD", &["EUR", "GBP"])? {
    println!("{}/{}: {} at {:?}", r.source, r.target, r.rate, r.time);
}

let rate = client.get_rate("USD", "EUR")?;
println!("USD/EUR: {:?} at {:?}", rate.rate, rate.date);
```

### `convert(from, to, amount)` and `convert_on_date(from, to, amount, date)`

Convert at the current rate, or at the rate on a past date (`YYYY-MM-DD`, last 365 days; the closest earlier point is used when the date has none). `convert_on_date` needs a paid plan.

```rust
let result = client.convert("USD", "EUR", 250.0)?;
println!("Rate: {:?}  Result: {:?}", result.rate, result.result);

let past = client.convert_on_date("USD", "EUR", 250.0, "2026-09-15")?;
println!("{:?}: {:?} -> {:?}", past.date, past.rate, past.result);
```

### `for_date(date, base, symbols)`

Paid plan. Rates on a past date (`YYYY-MM-DD`, last 365 days). `symbols` is required, and each symbol costs one request. Dates older than 30 days resolve against the weekly 1-year series, so you get the closest earlier weekly point.

```rust
let rates = client.for_date("2026-10-01", "USD", Some(&["EUR", "GBP"]))?;
println!("Date: {:?}", rates.date);
println!("Rates: {:?}", rates.rates);
```

### `time_series(start_date, end_date, base, symbols)`

Paid plan. Rates across a date range, keyed by date then currency. The start date must be within the last 365 days; `symbols` is required, one request per symbol.

```rust
let series = client.time_series(
    "2026-10-01",
    "2026-10-15",
    "USD",
    Some(&["EUR", "GBP"]),
)?;
for (date, day_rates) in series.rates.unwrap_or_default() {
    println!("{date}: {day_rates:?}");
}
```

### `get_historical_rates(source, target, period)`

Paid plan. A pair's history over a preset period: `"1d"`, `"7d"`, `"30d"` or `"1y"` (daily points; weekly for `"1y"`). Returns a `Vec<HistoricalRatePoint>` (`date`, `rate`, `timestamp` in Unix milliseconds), oldest first.

```rust
let history = client.get_historical_rates("USD", "EUR", "30d")?;
for point in history.rates.unwrap_or_default() {
    println!("{}: {:?}", point.date.unwrap_or_default(), point.rate);
}
```

### `symbols()`

Every supported currency, no key needed. `symbols` maps each code to a `{"code","name","symbol"}` JSON object; `currencies` is the typed list in the API's order.

```rust
let symbols = AllRatesToday::keyless().symbols()?;
println!("{:?} currencies", symbols.count);
for c in symbols.currencies.unwrap_or_default() {
    println!("{:?} {:?} {:?}", c.code, c.name, c.symbol);
}
```

### `official_rates(bank, source, target)` and `official_sources()`

The latest official table published by a central bank or tax authority (121 central banks + 3 tax authorities: HMRC, US Treasury, Swiss BAZG). These are the fixed, citable rates for invoicing, tax, customs and accounting, not live market rates. No key needed. Cite `rate_date`, not today's date.

```rust
let keyless = AllRatesToday::keyless();

// One pair (cross-computed inside the bank's table when not published directly)
let ecb = keyless.official_rates("ecb", Some("EUR"), Some("USD"))?;
println!("ECB {}: 1 EUR = {:?} USD ({:?})", ecb.rate_date, ecb.rate, ecb.rate_type);

// Full published table
let table = keyless.official_rates("ecb", None, None)?;
for row in table.rates.unwrap_or_default() {
    println!("{} {} {} {}", row.base, row.quote, row.rate_type, row.value);
}

// Every source with its latest publication date
for s in keyless.official_sources()?.sources {
    println!("{} {} {:?} stale={:?}", s.code, s.name, s.latest, s.stale);
}
```

Dated and historical central-bank tables (`/api/v1/central-bank/{bank}/{date}`) need a paid plan; see [pricing](https://allratestoday.com/pricing).

### `with_base_url(api_key, base_url)`

Override the default host (`https://allratestoday.com`), for example to point at a test server.

```rust
let client = AllRatesToday::with_base_url(
    "art_live_your_api_key",
    "https://custom.example.com",
);
```

## 🗺️ Currencies covered

160+ currencies, including 🇺🇸 `USD`, 🇪🇺 `EUR`, 🇬🇧 `GBP` and 🇯🇵 `JPY`. Call `symbols()` for the authoritative, always-current list rather than hard-coding one.

## 🛡️ Error handling

Every method returns `Result<T, AllRatesTodayError>`. The enum has four variants:

| Variant | Description |
|---------|-------------|
| `HttpError(reqwest::Error)` | Network or transport-level failure (timeout, DNS, TLS) |
| `ApiError { status, message }` | Non-2xx HTTP response; `status` is the code, `message` the response body |
| `ParseError(String)` | The response body could not be deserialised, or held no rate |
| `InvalidArgument(String)` | Rejected before sending: missing key, missing symbols, bad or out-of-range date, bad source code |

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
    Err(AllRatesTodayError::InvalidArgument(msg)) => eprintln!("Invalid argument: {msg}"),
}
```

`AllRatesTodayError` implements `std::error::Error` and `Display`, and converts from `reqwest::Error` and `serde_json::Error`, so `?` works inside functions returning `Box<dyn Error>` or the crate's own `allratestoday::Result<T>` alias.

## 💡 Notes

- **Blocking only.** The client uses `reqwest::blocking` (10-second timeout), so calls must not be made from inside an async runtime's worker thread — wrap them in `spawn_blocking` if you are on Tokio.
- **Optional fields.** Most response fields are `Option<_>` so an evolving API payload never fails to deserialise. Unwrap or pattern-match rather than assuming a value is present.
- **Request cost.** `for_date` and `time_series` make one request per symbol, because the history endpoint takes one target per call.
- **Tests.** `cargo test` runs the unit tests against a local mock server using response bodies captured from the live API; set `ALLRATES_API_KEY` to also run the live smoke test (4 keyed requests; on a free key the two history calls are expected to return 403 `historical_requires_paid_plan`).

## 🤖 Use it from an AI agent

The same rates are available to coding agents and LLM tools without a second integration:

- **Claude Code plugin** (no key): `/plugin marketplace add AllRates-Today/claude-code-plugin` then `/plugin install allratestoday@allratestoday`
- **Hosted MCP endpoint**: `https://allratestoday.com/api/mcp` — Streamable HTTP, keyless, nothing to install
- **Local MCP servers**: `npx -y @allratestoday/mcp-server` (live mid-market rates) · `npx -y @allratestoday/central-bank-mcp` (official central-bank rates)
- **Trial key for agents, no signup**: `POST https://allratestoday.com/api/agent/keys` (send `Content-Type: application/json`) returns a 100-call key valid 7 days for latest rates and latest central-bank tables (history and dated tables are paid-plan only); both MCP servers expose it as `request_trial_key`
- **Daily Reference Rate** (first published 2026-10-12): one fixed USD table per business day, the 16:00 London median of minute samples, never revised — keyless at `https://allratestoday.com/api/open/reference-rate`, `get_reference_rate` in the MCP server
- Machine-readable site guide: [llms.txt](https://allratestoday.com/llms.txt)

## 🔗 Links

- **Website:** [allratestoday.com](https://allratestoday.com)
- **API docs:** [allratestoday.com/docs](https://allratestoday.com/docs)
- **Free API key:** [allratestoday.com/register](https://allratestoday.com/register)
- **Crate:** [crates.io/crates/allratestoday](https://crates.io/crates/allratestoday)
- **Reference docs:** [docs.rs/allratestoday](https://docs.rs/allratestoday)
- **Repository:** [github.com/AllRates-Today/allratestoday-rust](https://github.com/AllRates-Today/allratestoday-rust)
- **Status:** [allratestoday.com/status](https://allratestoday.com/status)
- **Support:** [allratestoday.com/contact](https://allratestoday.com/contact)
- **Dashboard:** [allratestoday.com/profile](https://allratestoday.com/profile)

## 📜 License

MIT — see [LICENSE](LICENSE).
