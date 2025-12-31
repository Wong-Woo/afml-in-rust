use crate::config;
use crate::base::{Bar, read_trades};
use crate::ch2::time_bar::compute_time_bars;
use std::error::Error;
use chrono::{Utc, DateTime};
use plotters::prelude::*;

// Returns timestamps where CUSUM filter triggers
// h: threshold
pub fn compute_cusum_events(bars: &[Bar], h: f64) -> Vec<(DateTime<Utc>, f64)> {
    let mut events = Vec::new();
    let mut s_pos = 0.0;
    let mut s_neg = 0.0;

    for i in 1..bars.len() {
        let prev_close = bars[i-1].close;
        let curr_close = bars[i].close;
        
        // Log return
        let r_t = if prev_close > 0.0 {
            (curr_close / prev_close).ln()
        } else {
            0.0
        };

        // Symmetric CUSUM Filter
        // S_t^+ = max(0, S_{t-1}^+ + r_t)
        // S_t^- = min(0, S_{t-1}^- + r_t)
        
        s_pos = (s_pos + r_t).max(0.0);
        s_neg = (s_neg + r_t).min(0.0);

        if s_pos >= h {
            events.push((bars[i].time, bars[i].close));
            s_pos = 0.0; // Reset
        } else if s_neg <= -h {
            events.push((bars[i].time, bars[i].close));
            s_neg = 0.0; // Reset
        }
    }
    events
}

pub fn draw_cusum_filter() -> Result<(), Box<dyn Error>> {
    let file_path = config::TARDIS_CSV_PATH;
    println!("Reading trades from {}...", file_path);
    let trades = read_trades(file_path)?;
    println!("Read {} trades.", trades.len());

    // 1. Compute Time Bars (15 minutes)
    let time_interval_minutes = 15;
    println!("Computing {} minute time bars...", time_interval_minutes);
    let time_bars = compute_time_bars(&trades, time_interval_minutes);
    println!("Generated {} time bars.", time_bars.len());

    // 2. Calculate Threshold (h) based on Volatility
    let returns: Vec<f64> = time_bars.windows(2).map(|w| {
        let prev = w[0].close;
        let curr = w[1].close;
        if prev > 0.0 { (curr / prev).ln() } else { 0.0 }
    }).collect();

    let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
    let variance = returns.iter().map(|r| (r - mean_return).powi(2)).sum::<f64>() / (returns.len() - 1) as f64;
    let std_dev = variance.sqrt();

    let threshold = std_dev; // Use 1 std dev as threshold
    println!("Using threshold h = {:.6} (1 std dev of 15m returns)", threshold);

    // 3. Run CUSUM Filter
    let events = compute_cusum_events(&time_bars, threshold);
    println!("Detected {} CUSUM events.", events.len());

    // 4. Draw Chart
    let output_path = "src/ch2/result/cusum_filter.png";
    println!("Drawing chart to {}...", output_path);
    draw_cusum_chart(&time_bars, &events, output_path)?;
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

fn draw_cusum_chart(
    time_bars: &[Bar],
    events: &[(DateTime<Utc>, f64)],
    output_path: &str,
) -> Result<(), Box<dyn Error>> {
    let root = BitMapBackend::new(output_path, (1280, 768)).into_drawing_area();
    root.fill(&WHITE)?;

    let min_time = time_bars.first().unwrap().time;
    let max_time = time_bars.last().unwrap().time;
    
    let min_price = time_bars.iter().map(|b| b.low).fold(f64::INFINITY, f64::min);
    let max_price = time_bars.iter().map(|b| b.high).fold(f64::NEG_INFINITY, f64::max);

    let mut chart = ChartBuilder::on(&root)
        .caption("CUSUM Filter Events (Orange Dots) on 15m Time Bars", ("sans-serif", 30).into_font())
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

    // 2. Draw CUSUM Events
    chart.draw_series(
        events.iter().map(|(time, price)| {
            Circle::new(
                (*time, *price),
                5,
                RGBColor(255, 165, 0).filled(), // Orange
            )
        })
    )?
    .label("CUSUM Event")
    .legend(|(x, y)| Circle::new((x + 10, y), 5, RGBColor(255, 165, 0).filled()));

    chart.configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    Ok(())
}
