# AFML in Rust

This project implements concepts from "Advances in Financial Machine Learning" using Rust.

## Usage

Run the project using `cargo run` followed by a command to execute specific examples.

```bash
cargo run -- <command>
```

### Available Commands

| Command | Description |
|---------|-------------|
| `time` | Generate and plot Time Bars (15-minute intervals). |
| `tick` | Generate and plot Tick Bars. |
| `volume` | Generate and plot Volume Bars. |
| `dollar` | Generate and plot Dollar Bars. |
| `imbalance` | Generate and plot Tick Imbalance Bars (overlaid on Time Bars). |
| `compare` | Compare statistical distributions of different bar types. |
| `all` | Run all of the above examples sequentially. |

### Examples

**Generate Tick Imbalance Bars:**
```bash
cargo run -- imbalance
```

**Compare Distributions:**
```bash
cargo run -- compare
```

**Run Everything:**
```bash
cargo run -- all
```

## Project Structure

- `src/base`: Common data structures and utilities (Trade, Bar, CSV reading).
- `src/ch2`: Chapter 2 implementations (Financial Data Structures).
  - `time_bar.rs`: Time Bars
  - `tick_bar.rs`: Tick Bars
  - `volume_bar.rs`: Volume Bars
  - `dollar_bar.rs`: Dollar Bars
  - `tick_imbalance_bar.rs`: Tick Imbalance Bars
  - `compare_distribution.rs`: Statistical comparison
- `data/`: Input CSV data files.
