mod config;
pub mod base;
mod ch2;

fn main() {
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

    println!("\n--- Compare Distribution ---");
    if let Err(e) = ch2::run_compare_distribution() {
        eprintln!("Error in Compare Distribution: {}", e);
    }
}
