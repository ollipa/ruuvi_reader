# RuuviTag sensory data reader

[![Crates.io](https://img.shields.io/crates/v/ruuvi_reader)](https://crates.io/crates/ruuvi_reader)
[![Crates.io](https://img.shields.io/crates/l/ruuvi_reader)](./LICENSE)
[![CI](https://github.com/ollipa/ruuvi_reader/workflows/CI/badge.svg)](https://github.com/ollipa/ruuvi_reader/actions?query=workflow%3ACI)
[![docs](https://img.shields.io/badge/docs-ruuvi_reader%20-blue)](https://docs.rs/ruuvi_reader/0.1.0/ruuvi_reader/)

A library to collect sensory data from RuuviTag sensors using Bluetooth LE. In practice just a convenience wrapper over `btleplug` and `ruuvi_sensor_protocol` crates.

## Usage

A minimal example to scan sensory data and print it to stdout.

```rust
use ruuvi_reader::BleAdapter;

fn main() {
    let adapter = BleAdapter::connect().unwrap();
    let results = adapter.start_scan().unwrap();
    for result in results {
        match result {
            Ok(data) => println!("{:?}", data),
            Err(err) => eprintln!("{}", err),
        }
    }
}
```
