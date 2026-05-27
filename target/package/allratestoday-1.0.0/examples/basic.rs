//! Basic usage example for the AllRatesToday Rust SDK.
//!
//! Run with:
//!   cargo run --example basic
//!
//! Make sure the `ALLRATESTODAY_API_KEY` environment variable is set:
//!   export ALLRATESTODAY_API_KEY="art_live_your_api_key"

use allratestoday::AllRatesToday;

fn main() {
    // Read API key from environment
    let api_key = std::env::var("ALLRATESTODAY_API_KEY")
        .expect("Set the ALLRATESTODAY_API_KEY environment variable");

    let client = AllRatesToday::new(&api_key);

    // -----------------------------------------------------------------------
    // 1. Get latest exchange rates
    // -----------------------------------------------------------------------
    println!("--- Latest Rates (USD -> EUR, GBP, JPY) ---");
    match client.latest("USD", Some(&["EUR", "GBP", "JPY"])) {
        Ok(resp) => {
            println!("  Base: {:?}", resp.base);
            println!("  Date: {:?}", resp.date);
            if let Some(rates) = &resp.rates {
                for (code, rate) in rates {
                    println!("  {code}: {rate}");
                }
            }
        }
        Err(e) => eprintln!("  Error: {e}"),
    }

    // -----------------------------------------------------------------------
    // 2. Convert currency
    // -----------------------------------------------------------------------
    println!("\n--- Convert 250 USD to EUR ---");
    match client.convert("USD", "EUR", 250.0) {
        Ok(resp) => {
            println!("  Rate:   {:?}", resp.rate);
            println!("  Result: {:?}", resp.result);
        }
        Err(e) => eprintln!("  Error: {e}"),
    }

    // -----------------------------------------------------------------------
    // 3. Historical rates for a specific date
    // -----------------------------------------------------------------------
    println!("\n--- Historical Rates (2025-01-15, USD) ---");
    match client.for_date("2025-01-15", "USD", Some(&["EUR", "GBP"])) {
        Ok(resp) => {
            println!("  Date: {:?}", resp.date);
            if let Some(rates) = &resp.rates {
                for (code, rate) in rates {
                    println!("  {code}: {rate}");
                }
            }
        }
        Err(e) => eprintln!("  Error: {e}"),
    }

    // -----------------------------------------------------------------------
    // 4. Time series
    // -----------------------------------------------------------------------
    println!("\n--- Time Series (2025-01-01 to 2025-01-07, USD -> EUR) ---");
    match client.time_series("2025-01-01", "2025-01-07", "USD", Some(&["EUR"])) {
        Ok(resp) => {
            if let Some(rates) = &resp.rates {
                let mut dates: Vec<&String> = rates.keys().collect();
                dates.sort();
                for date in dates {
                    println!("  {date}: {:?}", rates[date]);
                }
            }
        }
        Err(e) => eprintln!("  Error: {e}"),
    }

    // -----------------------------------------------------------------------
    // 5. List all supported symbols
    // -----------------------------------------------------------------------
    println!("\n--- Supported Symbols (first 5) ---");
    match client.symbols() {
        Ok(resp) => {
            if let Some(symbols) = &resp.symbols {
                for (i, (code, info)) in symbols.iter().enumerate() {
                    if i >= 5 {
                        println!("  ... and {} more", symbols.len() - 5);
                        break;
                    }
                    println!("  {code}: {info}");
                }
            }
        }
        Err(e) => eprintln!("  Error: {e}"),
    }

    // -----------------------------------------------------------------------
    // 6. Single pair rate
    // -----------------------------------------------------------------------
    println!("\n--- Single Rate (USD -> EUR) ---");
    match client.get_rate("USD", "EUR") {
        Ok(resp) => {
            println!("  From: {:?}", resp.from);
            println!("  To:   {:?}", resp.to);
            println!("  Rate: {:?}", resp.rate);
            println!("  Date: {:?}", resp.date);
        }
        Err(e) => eprintln!("  Error: {e}"),
    }

    // -----------------------------------------------------------------------
    // 7. Historical rates with preset period
    // -----------------------------------------------------------------------
    println!("\n--- Historical Rates (USD -> EUR, 7d) ---");
    match client.get_historical_rates("USD", "EUR", "7d") {
        Ok(resp) => {
            println!("  Period: {:?}", resp.period);
            if let Some(points) = &resp.rates {
                for point in points {
                    println!(
                        "  {}: {:?}",
                        point.date.as_deref().unwrap_or("?"),
                        point.rate
                    );
                }
            }
        }
        Err(e) => eprintln!("  Error: {e}"),
    }
}
