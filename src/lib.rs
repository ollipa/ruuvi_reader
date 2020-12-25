use std::sync::mpsc::Receiver;

use btleplug::api::{BDAddr, Central, CentralEvent, Peripheral};
use btleplug::bluez::adapter::ConnectedAdapter;
use btleplug::bluez::manager::Manager;

pub use btleplug::Error as BleError;
pub use ruuvi_sensor_protocol::{ParseError, SensorValues};

/// Represents a physical bluetooth interface.
pub struct BleAdapter {
    adapter: ConnectedAdapter,
}

impl BleAdapter {
    /// Connects to an available Bluetooth adapter.
    pub fn connect() -> Result<Self, BleError> {
        let manager = Manager::new()?;
        let adapter = manager.adapters()?.pop().ok_or(BleError::DeviceNotFound)?;
        let adapter = adapter.connect()?;
        // Passive scanning
        adapter.active(false);
        // Receving duplicates can be useful when using scan to collect information from
        // beacons that update data frequently.
        adapter.filter_duplicates(false);
        Ok(BleAdapter { adapter })
    }

    /// Starts scanning for RuuviTags (BLE devices) and returns an iterator.
    pub fn start_scan(self) -> Result<ScanResults, BleError> {
        self.adapter.start_scan()?;
        // First call to event_receiver should always return Some.
        let receiver = self.adapter.event_receiver().unwrap();
        Ok(ScanResults {
            adapter: self.adapter,
            receiver,
        })
    }
}

/// An iterator over scan results that yields RuuviTag sensor value or parsing error.
/// Parsing errors should not occur normally but they can be useful for troubleshooting
/// issues with data collection.
pub struct ScanResults {
    adapter: ConnectedAdapter,
    receiver: Receiver<CentralEvent>,
}

impl Iterator for ScanResults {
    type Item = Result<SensorValues, ParseError>;

    fn next(&mut self) -> Option<Result<SensorValues, ParseError>> {
        loop {
            let event = self.receiver.iter().next();
            if let Some(event) = event {
                match event {
                    CentralEvent::DeviceDiscovered(address)
                    | CentralEvent::DeviceUpdated(address) => {
                        match parse_sensor_data(&self.adapter, address) {
                            Some(data) => return Some(data),
                            None => continue,
                        }
                    }
                    _ => continue,
                }
            };
            return None;
        }
    }
}

fn parse_sensor_data(
    adapter: &ConnectedAdapter,
    address: BDAddr,
) -> Option<Result<SensorValues, ParseError>> {
    let data = adapter
        .peripheral(address)
        .map(|peripheral| peripheral.properties().manufacturer_data)
        .flatten();

    if let Some(data) = data {
        if data.len() > 2 {
            let id = ((data[1] as u16) << 8) | data[0] as u16;
            return match SensorValues::from_manufacturer_specific_data(id, &data[2..]) {
                Ok(sensor_values) => Some(Ok(sensor_values)),
                Err(err) => match err {
                    // Ignore data from unknown devices
                    ParseError::UnknownManufacturerId(_) => None,
                    err => Some(Err(err)),
                },
            };
        }
    }
    None
}
