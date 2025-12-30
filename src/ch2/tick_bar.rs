use crate::config;
use crate::base::{Bar, read_trades, Trade, draw_chart_file};
use std::error::Error;
use chrono::{Utc, TimeZone};

pub fn compute_tick_bars(trades: &[Trade], interval_trades: usize) -> Vec<Bar> {
    if trades.is_empty() {
        return Vec::new();
    }

    let mut bars = Vec::new();

    let first_trade = &trades[0];

    let mut current_interval_start: usize = 0;
    let mut current_interval_end = current_interval_start + interval_trades;

    let mut open = first_trade.price;
    let mut high = first_trade.price;
    let mut low = first_trade.price;
    let mut close = first_trade.price;
    let mut volume = 0.0;
    let mut has_data = false;

    for (i, trade) in trades.iter().enumerate() {
        if i >= current_interval_end {
            if has_data {
                bars.push(Bar {
                    time: Utc.timestamp_micros(trades[current_interval_start].timestamp as i64).unwrap(),
                    open,
                    high,
                    low,
                    close,
                    volume,
                });
            }

            while i >= current_interval_end {
                current_interval_start = current_interval_end;
                current_interval_end += interval_trades;
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

    if has_data {
        bars.push(Bar {
            time: Utc.timestamp_micros(trades[current_interval_start].timestamp as i64).unwrap(),
            open,
            high,
            low,
            close,
            volume,
        });
    }

    bars
}

pub fn draw_tick_bar() -> Result<(), Box<dyn Error>> {
    let file_path = config::TARDIS_CSV_PATH;
    println!("Reading trades from {}...", file_path);
    let trades = read_trades(file_path)?;
    println!("Read {} trades.", trades.len());

    let interval_trades = 10000;
    println!("Computing {} tick as a bar...", interval_trades);
    let bars = compute_tick_bars(&trades, interval_trades);
    println!("Generated {} bars.", bars.len());

    let output_path = "src/ch2/result/tick_bars.png";
    println!("Drawing chart to {}...", output_path);
    draw_chart_file("Tick Bar (BTCUSDT)", &bars, output_path)?;

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
