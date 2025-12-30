use std::error::Error;
use std::path::Path;
use serde::Deserialize;
use chrono::{DateTime, Utc};
use plotters::prelude::*;

#[derive(Debug, Deserialize)]
pub struct Trade {
    pub exchange: String,
    pub symbol: String,
    pub timestamp: u64, // microseconds
    pub local_timestamp: u64,
    pub id: u64,
    pub side: String,
    pub price: f64,
    pub amount: f64,
}

#[derive(Debug, Clone)]
pub struct Bar {
    pub time: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

pub fn read_trades<P: AsRef<Path>>(path: P) -> Result<Vec<Trade>, Box<dyn Error>> {
    let mut rdr = csv::Reader::from_path(path)?;
    let mut trades = Vec::new();
    for result in rdr.deserialize() {
        let trade: Trade = result?;
        trades.push(trade);
    }
    Ok(trades)
}

pub fn draw_chart_file(title: &str,bars: &[Bar], output_path: &str) -> Result<(), Box<dyn Error>> {
    let root = BitMapBackend::new(output_path, (1024, 768)).into_drawing_area();
    root.fill(&WHITE)?;

    let min_time = bars.first().unwrap().time;
    let max_time = bars.last().unwrap().time;
    
    let min_price = bars.iter().map(|b| b.low).fold(f64::INFINITY, f64::min);
    let max_price = bars.iter().map(|b| b.high).fold(f64::NEG_INFINITY, f64::max);

    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", 50).into_font())
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(min_time..max_time, min_price..max_price)?;

    chart.configure_mesh().draw()?;

    chart.draw_series(
        bars.iter().map(|bar| {
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
        }),
    )?;

    root.present()?;
    println!("Chart saved to {}", output_path);

    Ok(())
}