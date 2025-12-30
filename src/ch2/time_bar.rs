use crate::config;
use crate::base::{Bar, read_trades, Trade, draw_chart_file};
use std::error::Error;
use chrono::{Utc, TimeZone};

pub fn compute_time_bars(trades: &[Trade], interval_minutes: i64) -> Vec<Bar> {
    if trades.is_empty() {
        return Vec::new();
    }

    let mut bars = Vec::new();
    let interval_micros = interval_minutes as u64 * 60 * 1_000_000;

    let first_trade = &trades[0];
    let start_time = first_trade.timestamp;
    
    // Align start time to the interval
    let mut current_interval_start = start_time - (start_time % interval_micros);
    let mut current_interval_end = current_interval_start + interval_micros;

    let mut open = first_trade.price;
    let mut high = first_trade.price;
    let mut low = first_trade.price;
    let mut close = first_trade.price;
    let mut volume = 0.0;
    let mut has_data = false;

    for trade in trades {
        if trade.timestamp >= current_interval_end {
            // Close current bar
            if has_data {
                bars.push(Bar {
                    time: Utc.timestamp_micros(current_interval_start as i64).unwrap(),
                    open,
                    high,
                    low,
                    close,
                    volume,
                });
            }

            // Move to next interval(s)
            while trade.timestamp >= current_interval_end {
                current_interval_start = current_interval_end;
                current_interval_end += interval_micros;
            }

            // Reset for new bar
            open = trade.price;
            high = trade.price;
            low = trade.price;
            close = trade.price;
            volume = trade.amount;
            has_data = true;
        } else {
            // Update current bar
            high = high.max(trade.price);
            low = low.min(trade.price);
            close = trade.price;
            volume += trade.amount;
            has_data = true;
        }
    }

    // Push the last bar
    if has_data {
        bars.push(Bar {
            time: Utc.timestamp_micros(current_interval_start as i64).unwrap(),
            open,
            high,
            low,
            close,
            volume,
        });
    }

    bars
}

pub fn draw_time_bar() -> Result<(), Box<dyn Error>> {
    let file_path = config::TARDIS_CSV_PATH;
    println!("Reading trades from {}...", file_path);
    let trades = read_trades(file_path)?;
    println!("Read {} trades.", trades.len());

    let interval_minutes = 15;
    println!("Computing {} minute time bars...", interval_minutes);
    let bars = compute_time_bars(&trades, interval_minutes);
    println!("Generated {} bars.", bars.len());

    let output_path = "src/ch2/result/time_bars.png";
    println!("Drawing chart to {}...", output_path);
    draw_chart_file("Time Bar (BTCUSDT)", &bars, output_path)?;

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
