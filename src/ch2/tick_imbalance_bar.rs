use crate::config;
use crate::base::{Bar, read_trades, Trade, draw_chart_file};
use crate::ch2::time_bar::compute_time_bars;
use std::error::Error;
use chrono::{Utc, TimeZone};
use plotters::prelude::*;

pub fn compute_tick_imbalance_bars(trades: &[Trade], initial_expected_ticks: f64) -> Vec<Bar> {
    if trades.is_empty() {
        return Vec::new();
    }

    let mut bars = Vec::new();
    let mut current_imbalance: f64 = 0.0;
    let mut current_ticks: f64 = 0.0;
    
    let mut open = trades[0].price;
    let mut high = trades[0].price;
    let mut low = trades[0].price;
    let mut close = trades[0].price;
    let mut volume = 0.0;
    let mut start_timestamp = trades[0].timestamp;
    
    let mut prev_price = trades[0].price;
    let mut prev_tick_rule = 1.0; // Init as buy
    
    let mut is_new_bar = true;

    // EWMA parameters
    let alpha = 0.05; // Smoothing factor (approx 20 bars)
    let mut ewma_expected_ticks = initial_expected_ticks;
    let mut ewma_expected_imbalance_per_tick: f64 = 0.5; // Initial guess for |2P[b=1]-1|

    for trade in trades {
        if is_new_bar {
            open = trade.price;
            high = trade.price;
            low = trade.price;
            // close is updated every trade
            volume = 0.0;
            start_timestamp = trade.timestamp;
            is_new_bar = false;
            current_ticks = 0.0;
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

        // 2. Accumulate Imbalance
        current_imbalance += tick_rule;
        current_ticks += 1.0;

        // Update Bar stats
        high = high.max(trade.price);
        low = low.min(trade.price);
        close = trade.price;
        volume += trade.amount;

        // 3. Check Threshold
        // Threshold = E[T] * |2P[b=1] - 1|
        let threshold = ewma_expected_ticks * ewma_expected_imbalance_per_tick.abs();
        
        if current_imbalance.abs() >= threshold {
            bars.push(Bar {
                time: Utc.timestamp_micros(trade.timestamp as i64).unwrap(), // Use current trade time (end of bar)
                open,
                high,
                low,
                close,
                volume,
            });

            // Update EWMA
            ewma_expected_ticks = alpha * current_ticks + (1.0 - alpha) * ewma_expected_ticks;
            
            let current_imbalance_per_tick = current_imbalance / current_ticks;
            ewma_expected_imbalance_per_tick = alpha * current_imbalance_per_tick + (1.0 - alpha) * ewma_expected_imbalance_per_tick;

            // Reset
            current_imbalance = 0.0;
            is_new_bar = true;
        }
    }
    
    bars
}

pub fn draw_tick_imbalance_bar() -> Result<(), Box<dyn Error>> {
    let file_path = config::TARDIS_CSV_PATH;
    println!("Reading trades from {}...", file_path);
    let trades = read_trades(file_path)?;
    println!("Read {} trades.", trades.len());

    // 1. Compute Time Bars (for background context)
    let time_interval_minutes = 15;
    println!("Computing {} minute time bars...", time_interval_minutes);
    let time_bars = compute_time_bars(&trades, time_interval_minutes);
    println!("Generated {} time bars.", time_bars.len());

    // 2. Compute Tick Imbalance Bars
    // Initial guess for expected ticks per bar
    // Lower initial guess to generate more bars initially
    let initial_expected_ticks = trades.len() as f64 / 1000.0; 
    
    println!("Computing Tick Imbalance Bars with dynamic threshold (init T={})...", initial_expected_ticks);
    let imbalance_bars = compute_tick_imbalance_bars(&trades, initial_expected_ticks);
    println!("Generated {} tick imbalance bars.", imbalance_bars.len());

    let output_path = "src/ch2/result/tick_imbalance_bars.png";
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
        .caption("Tick Imbalance Bars (Red Dots) vs 15m Time Bars (Candle)", ("sans-serif", 30).into_font())
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

    // 2. Draw Tick Imbalance Bars as Points
    chart.draw_series(
        imbalance_bars.iter().map(|b| {
            Circle::new(
                (b.time, b.close),
                4,
                BLUE.filled(),
            )
        })
    )?
    .label("Tick Imbalance Bar")
    .legend(|(x, y)| Circle::new((x + 10, y), 4, BLUE.filled()));

    chart.configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    Ok(())
}
