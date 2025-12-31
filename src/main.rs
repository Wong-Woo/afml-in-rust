use std::env;

mod config;
pub mod base;
mod ch2;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        return;
    }

    let command = &args[1];

    match command.as_str() {
        "time" => {
            println!("\n--- Time Bar ---");
            if let Err(e) = ch2::draw_time_bar() {
                eprintln!("Error in Time Bar: {}", e);
            }
        }
        "tick" => {
            println!("\n--- Tick Bar ---");
            if let Err(e) = ch2::draw_tick_bar() {
                eprintln!("Error in Tick Bar: {}", e);
            }
        }
        "volume" => {
            println!("\n--- Volume Bar ---");
            if let Err(e) = ch2::draw_volume_bar() {
                eprintln!("Error in Volume Bar: {}", e);
            }
        }
        "dollar" => {
            println!("\n--- Dollar Bar ---");
            if let Err(e) = ch2::draw_dollar_bar() {
                eprintln!("Error in Dollar Bar: {}", e);
            }
        }
        "imbalance" => {
            println!("\n--- Tick Imbalance Bar ---");
            if let Err(e) = ch2::draw_tick_imbalance_bar() {
                eprintln!("Error in Tick Imbalance Bar: {}", e);
            }
        }
        "volume_imbalance" => {
            println!("\n--- Volume Imbalance Bar ---");
            if let Err(e) = ch2::draw_volume_imbalance_bar() {
                eprintln!("Error in Volume Imbalance Bar: {}", e);
            }
        }
        "dollar_imbalance" => {
            println!("\n--- Dollar Imbalance Bar ---");
            if let Err(e) = ch2::draw_dollar_imbalance_bar() {
                eprintln!("Error in Dollar Imbalance Bar: {}", e);
            }
        }
        "cusum" => {
            println!("\n--- CUSUM Filter ---");
            if let Err(e) = ch2::draw_cusum_filter() {
                eprintln!("Error in CUSUM Filter: {}", e);
            }
        }
        "compare" => {
            println!("\n--- Compare Distribution ---");
            if let Err(e) = ch2::run_compare_distribution() {
                eprintln!("Error in Compare Distribution: {}", e);
            }
        }
        "all" => {
            run_all();
        }
        _ => {
            println!("Unknown command: {}", command);
            print_usage();
        }
    }
}

fn run_all() {
    println!("AFML in Rust - Chapter Examples");

    println!("\n--- Time Bar ---");
    if let Err(e) = ch2::draw_time_bar() {
        eprintln!("Error in Time Bar: {}", e);
    }

    println!("\n--- Tick Bar ---");
    if let Err(e) = ch2::draw_tick_bar() {
        eprintln!("Error in Tick Bar: {}", e);
    }

    println!("\n--- Volume Bar ---");
    if let Err(e) = ch2::draw_volume_bar() {
        eprintln!("Error in Volume Bar: {}", e);
    }

    println!("\n--- Dollar Bar ---");
    if let Err(e) = ch2::draw_dollar_bar() {
        eprintln!("Error in Dollar Bar: {}", e);
    }

    println!("\n--- Tick Imbalance Bar ---");
    if let Err(e) = ch2::draw_tick_imbalance_bar() {
        eprintln!("Error in Tick Imbalance Bar: {}", e);
    }

    println!("\n--- Volume Imbalance Bar ---");
    if let Err(e) = ch2::draw_volume_imbalance_bar() {
        eprintln!("Error in Volume Imbalance Bar: {}", e);
    }

    println!("\n--- Dollar Imbalance Bar ---");
    if let Err(e) = ch2::draw_dollar_imbalance_bar() {
        eprintln!("Error in Dollar Imbalance Bar: {}", e);
    }

    println!("\n--- Compare Distribution ---");
    if let Err(e) = ch2::run_compare_distribution() {
        eprintln!("Error in Compare Distribution: {}", e);
    }
}

fn print_usage() {
    println!("Usage: cargo run -- <command>");
    println!("Commands:");
    println!("  time       - Generate Time Bars");
    println!("  tick       - Generate Tick Bars");
    println!("  volume     - Generate Volume Bars");
    println!("  dollar     - Generate Dollar Bars");
    println!("  imbalance         - Generate Tick Imbalance Bars");
    println!("  volume_imbalance  - Generate Volume Imbalance Bars");
    println!("  dollar_imbalance  - Generate Dollar Imbalance Bars");
    println!("  compare           - Compare Distributions");
    println!("  all        - Run all examples");
}
