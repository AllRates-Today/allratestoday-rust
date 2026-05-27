# allratestoday

[![Crates.io](https://img.shields.io/crates/v/allratestoday.svg)](https://crates.io/crates/allratestoday)
[![Docs.rs](https://docs.rs/allratestoday/badge.svg)](https://docs.rs/allratestoday)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Official Rust SDK for [AllRatesToday](https://allratestoday.com) -- real-time mid-market exchange rates for 160+ currencies.

## Installation

Add the crate to your project:

```bash
cargo add allratestoday
```

Or add it manually to your `Cargo.toml`:

```toml
[dependencies]
allratestoday = "1"
```

## Quick Start

```rust
use allratestoday::AllRatesToday;

fn main() {
    let client = AllRatesToday::new("art_live_your_api_key");

    // Get latest rates
    let rates = client.latest("USD", Some(&["EUR", "GBP"])).unwrap();
    println!("{:?}", rates);

    // Convert currency
    let result = client.convert("USD", "EUR", 100.0).unwrap();
    println!("100 USD = {:?} EUR", result.result);
}
```

## Authentication

Sign up at [allratestoday.com](https://allratestoday.com) to get your API key. Keys follow the format `art_live_...`.

The SDK sends the key as a Bearer token in the `Authorization` header.

## API Methods

### `latest(base, symbols)` -- Get Latest Rates

Fetch the most recent exchange rates for a base currency.

```rust
// All currencies
let rates = client.latest("USD", None)?;

// Specific currencies only
let rates = client.latest("USD", Some(&["EUR", "GBP", "JPY"]))?;
println!("Base: {:?}", rates.base);
println!("Rates: {:?}", rates.rates);
```

### `convert(from, to, amount)` -- Convert Currency

Convert an amount from one currency to another.

```rust
let result = client.convert("USD", "EUR", 250.0)?;
println!("Rate: {:?}", result.rate);
println!("Result: {:?}", result.result);
```

### `for_date(date, base, symbols)` -- Historical Rates

Get exchange rates for a specific historical date.

```rust
let rates = client.for_date("2025-01-15", "USD", Some(&["EUR", "GBP"]))?;
println!("Date: {:?}", rates.date);
println!("Rates: {:?}", rates.rates);
```

### `time_series(start_date, end_date, base, symbols)` -- Time Series

Get exchange rates over a date range.

```rust
let series = client.time_series(
    "2025-01-01",
    "2025-01-31",
    "USD",
    Some(&["EUR", "GBP"]),
)?;
for (date, day_rates) in series.rates.unwrap_or_default() {
    println!("{date}: {:?}", day_rates);
}
```

### `symbols()` -- List Supported Currencies

Retrieve all supported currency codes and names.

```rust
let symbols = client.symbols()?;
for (code, info) in symbols.symbols.unwrap_or_default() {
    println!("{code}: {info}");
}
```

### `get_rate(from, to)` -- Single Pair Rate

Get the exchange rate for a single currency pair.

```rust
let rate = client.get_rate("USD", "EUR")?;
println!("USD/EUR: {:?}", rate.rate);
```

### `get_historical_rates(source, target, period)` -- Preset Period Historical Rates

Get historical rates for a currency pair over a preset period.

Supported periods: `1d`, `7d`, `30d`, `1y`.

```rust
let history = client.get_historical_rates("USD", "EUR", "30d")?;
for point in history.rates.unwrap_or_default() {
    println!("{}: {:?}", point.date.unwrap_or_default(), point.rate);
}
```

## Error Handling

All methods return `Result<T, AllRatesTodayError>`. The error enum has three variants:

| Variant      | Description                                          |
|--------------|------------------------------------------------------|
| `HttpError`  | Network or transport-level failure (timeout, DNS, TLS) |
| `ApiError`   | Non-2xx HTTP response from the API                   |
| `ParseError` | Failed to deserialize the response body              |

```rust
use allratestoday::{AllRatesToday, AllRatesTodayError};

let client = AllRatesToday::new("art_live_your_api_key");

match client.latest("USD", None) {
    Ok(rates) => println!("{:?}", rates),
    Err(AllRatesTodayError::HttpError(e)) => eprintln!("Network error: {e}"),
    Err(AllRatesTodayError::ApiError { status, message }) => {
        eprintln!("API error {status}: {message}");
    }
    Err(AllRatesTodayError::ParseError(msg)) => eprintln!("Parse error: {msg}"),
}
```

## Custom Base URL

For testing or self-hosted instances, you can override the base URL:

```rust
let client = AllRatesToday::with_base_url(
    "art_live_your_api_key",
    "https://custom.example.com",
);
```

## Examples

Run the included example:

```bash
export ALLRATESTODAY_API_KEY="art_live_your_api_key"
cargo run --example basic
```

## License

This project is licensed under the [MIT License](LICENSE).

## Links

- Website: [allratestoday.com](https://allratestoday.com)
- API Documentation: [allratestoday.com/docs](https://allratestoday.com/docs)
- Crate: [crates.io/crates/allratestoday](https://crates.io/crates/allratestoday)
- Docs: [docs.rs/allratestoday](https://docs.rs/allratestoday)
- Repository: [github.com/allratestoday/allratestoday-rust](https://github.com/allratestoday/allratestoday-rust)
