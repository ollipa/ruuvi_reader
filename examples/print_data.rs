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
