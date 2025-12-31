use crate::config;
use crate::base::{read_trades, Bar};
use crate::ch2::time_bar::compute_time_bars;
use crate::ch2::tick_bar::compute_tick_bars;
use crate::ch2::volume_bar::compute_volume_bars;
use crate::ch2::dollar_bar::compute_dollar_bars;
use std::error::Error;
use plotters::prelude::*;

struct Stats {
    skewness: f64,
    kurtosis: f64,
}

fn compute_stats(data: &[f64]) -> Stats {
    let n = data.len() as f64;
    if n <= 2.0 { return Stats { skewness: 0.0, kurtosis: 0.0 }; }
    
    let mean = data.iter().sum::<f64>() / n;
    let variance = data.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0);
    let std_dev = variance.sqrt();
    
    let skewness = data.iter().map(|x| ((x - mean) / std_dev).powi(3)).sum::<f64>() / n;
    let kurtosis = data.iter().map(|x| ((x - mean) / std_dev).powi(4)).sum::<f64>() / n;
    
    Stats {
        skewness,
        kurtosis: kurtosis - 3.0, // Excess Kurtosis
    }
}

fn compute_log_returns(bars: &[Bar]) -> Vec<f64> {
    let mut returns = Vec::new();
    for i in 1..bars.len() {
        let prev_close = bars[i - 1].close;
        let curr_close = bars[i].close;
        if prev_close > 0.0 && curr_close > 0.0 {
            let r = (curr_close / prev_close).ln();
            returns.push(r);
        }
    }
    returns
}

fn standardize(data: &[f64]) -> Vec<f64> {
    let n = data.len() as f64;
    if n <= 1.0 { return data.to_vec(); }
    let mean = data.iter().sum::<f64>() / n;
    let variance = data.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0);
    let std = variance.sqrt();
    if std == 0.0 { return data.to_vec(); }
    data.iter().map(|x| (x - mean) / std).collect()
}

pub fn run_compare_distribution() -> Result<(), Box<dyn Error>> {
    let file_path = config::TARDIS_CSV_PATH;
    println!("Reading trades from {}...", file_path);
    let trades = read_trades(file_path)?;
    println!("Read {} trades.", trades.len());

    // 1. Compute Time Bars (e.g., 15 minutes)
    let time_interval_minutes = 15;
    println!("Computing {} minute time bars...", time_interval_minutes);
    let time_bars = compute_time_bars(&trades, time_interval_minutes);
    println!("Generated {} time bars.", time_bars.len());
    let num_time_bars = time_bars.len();

    // 2. Compute Tick Bars
    let total_trades = trades.len();
    let tick_interval = if num_time_bars > 0 {
        total_trades / num_time_bars
    } else {
        1000 // fallback
    };
    println!("Computing {} tick bars...", tick_interval);
    let tick_bars = compute_tick_bars(&trades, tick_interval);
    println!("Generated {} tick bars.", tick_bars.len());

    // 3. Compute Volume Bars
    let total_volume: f64 = trades.iter().map(|t| t.amount).sum();
    let volume_interval = if num_time_bars > 0 {
        total_volume / num_time_bars as f64
    } else {
        1000.0
    };
    println!("Computing {:.2} volume bars...", volume_interval);
    let volume_bars = compute_volume_bars(&trades, volume_interval);
    println!("Generated {} volume bars.", volume_bars.len());

    // 4. Compute Dollar Bars
    let total_dollar: f64 = trades.iter().map(|t| t.price * t.amount).sum();
    let dollar_interval = if num_time_bars > 0 {
        total_dollar / num_time_bars as f64
    } else {
        10000.0
    };
    println!("Computing {:.2} dollar bars...", dollar_interval);
    let dollar_bars = compute_dollar_bars(&trades, dollar_interval);
    println!("Generated {} dollar bars.", dollar_bars.len());

    // 5. Compute Returns & Stats
    let time_returns = compute_log_returns(&time_bars);
    let tick_returns = compute_log_returns(&tick_bars);
    let volume_returns = compute_log_returns(&volume_bars);
    let dollar_returns = compute_log_returns(&dollar_bars);
    
    let time_stats = compute_stats(&time_returns);
    let tick_stats = compute_stats(&tick_returns);
    let volume_stats = compute_stats(&volume_returns);
    let dollar_stats = compute_stats(&dollar_returns);

    println!("--- Statistics ---");
    println!("Time Bar: Skewness={:.4}, Excess Kurtosis={:.4}", time_stats.skewness, time_stats.kurtosis);
    println!("Tick Bar: Skewness={:.4}, Excess Kurtosis={:.4}", tick_stats.skewness, tick_stats.kurtosis);
    println!("Volume Bar: Skewness={:.4}, Excess Kurtosis={:.4}", volume_stats.skewness, volume_stats.kurtosis);
    println!("Dollar Bar: Skewness={:.4}, Excess Kurtosis={:.4}", dollar_stats.skewness, dollar_stats.kurtosis);

    let time_std = standardize(&time_returns);
    let tick_std = standardize(&tick_returns);
    let volume_std = standardize(&volume_returns);
    let dollar_std = standardize(&dollar_returns);

    // 6. Draw Histograms
    let output_tick = "src/ch2/result/compare_time_vs_tick.png";
    draw_distribution_chart(&time_std, &tick_std, &time_stats, &tick_stats, "Time Bar", "Tick Bar", output_tick)?;
    
    let output_volume = "src/ch2/result/compare_time_vs_volume.png";
    draw_distribution_chart(&time_std, &volume_std, &time_stats, &volume_stats, "Time Bar", "Volume Bar", output_volume)?;

    let output_dollar = "src/ch2/result/compare_time_vs_dollar.png";
    draw_distribution_chart(&time_std, &dollar_std, &time_stats, &dollar_stats, "Time Bar", "Dollar Bar", output_dollar)?;

    // Open files
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd").args(&["/C", "start", output_tick]).spawn()?;
        std::process::Command::new("cmd").args(&["/C", "start", output_volume]).spawn()?;
        std::process::Command::new("cmd").args(&["/C", "start", output_dollar]).spawn()?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(output_tick).spawn()?;
        std::process::Command::new("open").arg(output_volume).spawn()?;
        std::process::Command::new("open").arg(output_dollar).spawn()?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open").arg(output_tick).spawn()?;
        std::process::Command::new("xdg-open").arg(output_volume).spawn()?;
        std::process::Command::new("xdg-open").arg(output_dollar).spawn()?;
    }

    Ok(())
}

fn draw_distribution_chart(
    data1: &[f64],
    data2: &[f64],
    stats1: &Stats,
    stats2: &Stats,
    label1: &str,
    label2: &str,
    output_path: &str,
) -> Result<(), Box<dyn Error>> {
    let root = BitMapBackend::new(output_path, (1024, 768)).into_drawing_area();
    root.fill(&WHITE)?;

    let title = format!(
        "Return Distribution ({} K={:.2} vs {} K={:.2})",
        label1, stats1.kurtosis, label2, stats2.kurtosis
    );

    // Define range for standardized data (e.g., -5 to 5 sigma)
    let min_x = -5.0;
    let max_x = 5.0;
    
    // Scott's Rule for bin size: h = 3.5 * sigma / n^(1/3)
    // Since data is standardized, sigma ~ 1.0
    // bins = (max_x - min_x) / h = 10 / (3.5 / n^(1/3)) approx 2.85 * n^(1/3)
    let n = data1.len().max(data2.len()) as f64;
    let bins = if n > 0.0 {
        (2.85 * n.powf(1.0 / 3.0)).round() as usize
    } else {
        50
    };
    let bins = bins.max(20).min(200); // Clamp to reasonable range
    
    let step = (max_x - min_x) / bins as f64;
    
    // Compute histograms (Density)
    let compute_density = |data: &[f64]| -> Vec<(f64, f64)> {
        let mut hist = vec![0.0; bins];
        let mut count = 0.0;
        for &val in data {
            if val >= min_x && val < max_x {
                let idx = ((val - min_x) / step).floor() as usize;
                if idx < bins {
                    hist[idx] += 1.0;
                    count += 1.0;
                }
            }
        }
        // Normalize to PDF: sum(hist * step) = 1 => hist_val = count / (total * step)
        if count == 0.0 { return vec![]; }
        hist.into_iter()
            .enumerate()
            .map(|(i, c)| {
                let x = min_x + i as f64 * step;
                let y = c / (count * step); 
                (x, y)
            })
            .collect()
    };

    let hist1 = compute_density(data1);
    let hist2 = compute_density(data2);
    
    // Gaussian (Standard Normal) for reference
    let gaussian: Vec<(f64, f64)> = (0..bins)
        .map(|i| {
            let x = min_x + i as f64 * step;
            let y = (-0.5 * x * x).exp() / (2.0 * std::f64::consts::PI).sqrt();
            (x, y)
        })
        .collect();

    let max_y = hist1.iter().chain(hist2.iter()).map(|(_, y)| *y).fold(0.0, f64::max).max(0.5);

    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", 40).into_font())
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(min_x..max_x, 0.0..max_y * 1.1)?;

    chart.configure_mesh().draw()?;

    // Data 1 (Red)
    chart.draw_series(
        hist1.iter().map(|&(x, y)| {
            Rectangle::new(
                [(x, 0.0), (x + step, y)],
                RED.mix(0.3).filled(),
            )
        })
    )?
    .label(format!("{} (Skew={:.2}, Kurt={:.2})", label1, stats1.skewness, stats1.kurtosis))
    .legend(|(x, y)| Rectangle::new([(x, y - 5), (x + 10, y + 5)], RED.mix(0.3).filled()));

    // Data 2 (Blue)
    chart.draw_series(
        hist2.iter().map(|&(x, y)| {
            Rectangle::new(
                [(x, 0.0), (x + step, y)],
                BLUE.mix(0.3).filled(),
            )
        })
    )?
    .label(format!("{} (Skew={:.2}, Kurt={:.2})", label2, stats2.skewness, stats2.kurtosis))
    .legend(|(x, y)| Rectangle::new([(x, y - 5), (x + 10, y + 5)], BLUE.mix(0.3).filled()));
        
    // Gaussian (Black, Dashed)
    chart.draw_series(LineSeries::new(gaussian, &BLACK.mix(0.8)))?
        .label("Normal Dist")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLACK));

    chart.configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    Ok(())
}
