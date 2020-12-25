# RuuviTag sensory data reader

A library to collect sensory data from RuuviTag sensors using Bluetooth LE. In practice just a convience wrapper over `btleplug` and `ruuvi_sensor_protocol` crates.

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
