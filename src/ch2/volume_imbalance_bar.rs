use crate::config;
use crate::base::{Bar, read_trades, Trade};
use crate::ch2::time_bar::compute_time_bars;
use std::error::Error;
use chrono::{Utc, TimeZone};
use plotters::prelude::*;

pub fn compute_volume_imbalance_bars(trades: &[Trade], initial_expected_volume: f64) -> Vec<Bar> {
    if trades.is_empty() {
        return Vec::new();
    }

    let mut bars = Vec::new();
    let mut current_imbalance: f64 = 0.0;
    let mut current_volume: f64 = 0.0;
    
    let mut open = trades[0].price;
    let mut high = trades[0].price;
    let mut low = trades[0].price;
    let mut close;
    let mut volume = 0.0;
    
    let mut prev_price = trades[0].price;
    let mut prev_tick_rule = 1.0; // Init as buy
    
    let mut is_new_bar = true;

    // EWMA parameters
    let alpha = 0.05; // Smoothing factor
    let mut ewma_expected_volume = initial_expected_volume;
    let mut ewma_expected_imbalance_per_tick: f64 = 0.5; // Initial guess for |2P[b=1]-1|

    // For calculating imbalance per tick (or per unit), we need to track ticks or just use the ratio
    // In standard implementation, we track expected imbalance (theta).
    // Threshold = E[V] * |2P[b=1] - 1|
    // Here we update |2P[b=1] - 1| based on observed imbalance.

    for trade in trades {
        if is_new_bar {
            open = trade.price;
            high = trade.price;
            low = trade.price;
            volume = 0.0;
            is_new_bar = false;
            current_volume = 0.0;
        }

        // 1. Calculate Tick Rule
        let tick_rule = if trade.price > prev_price {
            1.0
        } else if trade.price < prev_price {
            -1.0
        } else {
            prev_tick_rule
        };
        prev_tick_rule = tick_rule;
        prev_price = trade.price;

        // 2. Accumulate Imbalance (Volume * Tick Rule)
        let signed_volume = tick_rule * trade.amount;
        current_imbalance += signed_volume;
        current_volume += trade.amount;

        // Update Bar stats
        high = high.max(trade.price);
        low = low.min(trade.price);
        close = trade.price;
        volume += trade.amount;

        // 3. Check Threshold
        // Threshold = E[V] * |2P[b=1] - 1|
        let threshold = ewma_expected_volume * ewma_expected_imbalance_per_tick.abs();
        
        if current_imbalance.abs() >= threshold {
            bars.push(Bar {
                time: Utc.timestamp_micros(trade.timestamp as i64).unwrap(),
                open,
                high,
                low,
                close,
                volume,
            });

            // Update EWMA
            ewma_expected_volume = alpha * current_volume + (1.0 - alpha) * ewma_expected_volume;
            
            // Update expected imbalance ratio. 
            // We can approximate |2P[b=1]-1| as |current_imbalance / current_volume|
            // Since current_imbalance = sum(b_i * v_i) and current_volume = sum(v_i)
            let observed_imbalance_ratio = if current_volume > 0.0 {
                current_imbalance / current_volume
            } else {
                0.0
            };
            
            ewma_expected_imbalance_per_tick = alpha * observed_imbalance_ratio + (1.0 - alpha) * ewma_expected_imbalance_per_tick;

            // Reset
            current_imbalance = 0.0;
            is_new_bar = true;
        }
    }
    
    bars
}

pub fn draw_volume_imbalance_bar() -> Result<(), Box<dyn Error>> {
    let file_path = config::TARDIS_CSV_PATH;
    println!("Reading trades from {}...", file_path);
    let trades = read_trades(file_path)?;
    println!("Read {} trades.", trades.len());

    // 1. Compute Time Bars (for background context)
    let time_interval_minutes = 15;
    println!("Computing {} minute time bars...", time_interval_minutes);
    let time_bars = compute_time_bars(&trades, time_interval_minutes);
    println!("Generated {} time bars.", time_bars.len());

    // 2. Compute Volume Imbalance Bars
    // Initial guess: Total Volume / 300
    let total_volume: f64 = trades.iter().map(|t| t.amount).sum();
    let initial_expected_volume = total_volume / 300.0;
    
    println!("Computing Volume Imbalance Bars with dynamic threshold (init V={:.2})...", initial_expected_volume);
    let imbalance_bars = compute_volume_imbalance_bars(&trades, initial_expected_volume);
    println!("Generated {} volume imbalance bars.", imbalance_bars.len());

    let output_path = "src/ch2/result/volume_imbalance_bars.png";
    println!("Drawing chart to {}...", output_path);
    
    draw_overlay_chart(&time_bars, &imbalance_bars, output_path)?;
    
    println!("Chart saved to {}", output_path);

    // Open file
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

fn draw_overlay_chart(
    time_bars: &[Bar],
    imbalance_bars: &[Bar],
    output_path: &str,
) -> Result<(), Box<dyn Error>> {
    let root = BitMapBackend::new(output_path, (1280, 768)).into_drawing_area();
    root.fill(&WHITE)?;

    let min_time = time_bars.first().unwrap().time;
    let max_time = time_bars.last().unwrap().time;
    
    let min_price = time_bars.iter().map(|b| b.low).fold(f64::INFINITY, f64::min);
    let max_price = time_bars.iter().map(|b| b.high).fold(f64::NEG_INFINITY, f64::max);

    let mut chart = ChartBuilder::on(&root)
        .caption("Volume Imbalance Bars (Blue Dots) vs 15m Time Bars (Candle)", ("sans-serif", 30).into_font())
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(min_time..max_time, min_price..max_price)?;

    chart.configure_mesh().draw()?;

    // 1. Draw Time Bars as CandleStick
    chart.draw_series(
        time_bars.iter().map(|bar| {
            CandleStick::new(
                bar.time,
                bar.open,
                bar.high,
                bar.low,
                bar.close,
                RGBColor(98, 209, 61).filled(),
                RGBColor(209, 61, 61).filled(),
                5,
            )
        })
    )?;

    // 2. Draw Imbalance Bars as Points
    chart.draw_series(
        imbalance_bars.iter().map(|b| {
            Circle::new(
                (b.time, b.close),
                4,
                BLUE.filled(),
            )
        })
    )?
    .label("Volume Imbalance Bar")
    .legend(|(x, y)| Circle::new((x + 10, y), 4, BLUE.filled()));

    chart.configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    Ok(())
}
