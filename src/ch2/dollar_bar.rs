use crate::config;
use crate::base::{Bar, read_trades, Trade, draw_chart_file};
use std::error::Error;
use chrono::{Utc, TimeZone};

pub fn compute_dollar_bars(trades: &[Trade], interval_dollar: f64) -> Vec<Bar> {
    if trades.is_empty() {
        return Vec::new();
    }

    let mut bars = Vec::new();
    let first_trade = &trades[0];

    let mut current_dollar_accumulated = 0.0;
    let mut current_bar_start_idx = 0;

    let mut open = first_trade.price;
    let mut high = first_trade.price;
    let mut low = first_trade.price;
    let mut close = first_trade.price;
    let mut volume = 0.0;
    let mut has_data = false;

    for (i, trade) in trades.iter().enumerate() {
        let dollar_value = trade.price * trade.amount;

        // Update current bar stats
        if !has_data {
            open = trade.price;
            high = trade.price;
            low = trade.price;
            close = trade.price;
            volume = trade.amount;
            has_data = true;
            current_bar_start_idx = i;
        } else {
            high = high.max(trade.price);
            low = low.min(trade.price);
            close = trade.price;
            volume += trade.amount;
        }

        current_dollar_accumulated += dollar_value;

        // Check if threshold reached
        if current_dollar_accumulated >= interval_dollar {
            bars.push(Bar {
                time: Utc.timestamp_micros(trades[current_bar_start_idx].timestamp as i64).unwrap(),
                open,
                high,
                low,
                close,
                volume,
            });

            // Reset
            current_dollar_accumulated = 0.0;
            has_data = false;
        }
    }

    // Optional: Push last partial bar
    if has_data {
        bars.push(Bar {
            time: Utc.timestamp_micros(trades[current_bar_start_idx].timestamp as i64).unwrap(),
            open,
            high,
            low,
            close,
            volume,
        });
    }

    bars
}

pub fn draw_dollar_bar() -> Result<(), Box<dyn Error>> {
    let file_path = config::TARDIS_CSV_PATH;
    println!("Reading trades from {}...", file_path);
    let trades = read_trades(file_path)?;
    println!("Read {} trades.", trades.len());

    let interval_dollar = 10_000_000.0; // Example dollar threshold (10M USDT)
    println!("Computing {} dollar value as a bar...", interval_dollar);
    let bars = compute_dollar_bars(&trades, interval_dollar);
    println!("Generated {} bars.", bars.len());

    let output_path = "src/ch2/result/dollar_bars.png";
    println!("Drawing chart to {}...", output_path);
    draw_chart_file("Dollar Bar (BTCUSDT)", &bars, output_path)?;

    // Try to open the file automatically
    #[cfg(target_os = "windows")]
    std::process::Command::new("cmd")
        .args(&["/C", "start", output_path])
        .spawn()?;

    #[cfg(target_os = "macos")]
    std::process::Command::new("open")
        .arg(output_path)
        .spawn()?;

    #[cfg(target_os = "linux")]
    std::process::Command::new("xdg-open")
        .arg(output_path)
        .spawn()?;

    Ok(())
}
