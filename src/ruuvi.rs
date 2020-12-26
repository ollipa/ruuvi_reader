use btleplug::api::{BDAddr, Central, Peripheral};
use btleplug::bluez::adapter::ConnectedAdapter;

use ruuvi_sensor_protocol::{
    Acceleration, BatteryPotential, Humidity, MeasurementSequenceNumber, MovementCounter, Pressure,
    Temperature, TransmitterPower,
};
use ruuvi_sensor_protocol::{ParseError, SensorValues};

/// Represents values read from RuuviTag sensor.
#[derive(Debug)]
pub struct SensorData {
    /// Sensor MAC address.
    pub mac_address: [u8; 6],
    /// Humidity in parts per million.
    pub humidity: u32,
    /// Temperature in milli-kelvins.
    pub temperature: u32,
    /// Pressure in pascals.
    pub pressure: u32,
    /// 3-dimensional acceleration vector, each component is in milli-G.
    pub acceleration: (i16, i16, i16),
    /// Battery potential in milli-volts.
    pub battery_potential: u16,
    /// Transmitter power in dBm.
    pub tx_power: Option<i8>,
    /// Movement counter.
    pub movement_counter: Option<u32>,
    /// Measurement sequence number.
    pub measurement_sequence_number: Option<u32>,
}

impl SensorData {
    pub fn new(
        mac_address: [u8; 6],
        humidity: u32,
        temperature: u32,
        pressure: u32,
        battery_potential: u16,
        acceleration: (i16, i16, i16),
    ) -> Self {
        Self {
            mac_address,
            humidity,
            temperature,
            pressure,
            acceleration,
            battery_potential,
            tx_power: None,
            movement_counter: None,
            measurement_sequence_number: None,
        }
    }

    pub fn temperature_as_millicelsius(&self) -> i32 {
        self.temperature as i32 - 273_150
    }

    pub fn mac_address_as_string(&self) -> String {
        self.mac_address
            .iter()
            .map(|x| format!("{:X?}", x))
            .rev()
            .collect::<Vec<String>>()
            .join(":")
    }
}

pub(crate) fn parse_sensor_data(
    adapter: &ConnectedAdapter,
    address: BDAddr,
) -> Option<Result<SensorData, ParseError>> {
    let data = adapter
        .peripheral(address)
        .map(|peripheral| peripheral.properties().manufacturer_data)
        .flatten();

    if let Some(data) = data {
        if data.len() > 2 {
            let id = ((data[1] as u16) << 8) | data[0] as u16;
            return match SensorValues::from_manufacturer_specific_data(id, &data[2..]) {
                Ok(sensor_values) => {
                    let humidity = sensor_values.humidity_as_ppm()?;
                    let temperature = sensor_values.temperature_as_millikelvins()?;
                    let pressure = sensor_values.pressure_as_pascals()?;
                    let acceleration = sensor_values.acceleration_vector_as_milli_g()?;
                    let battery_potential = sensor_values.battery_potential_as_millivolts()?;

                    let mut sensor_data = SensorData::new(
                        address.address,
                        humidity,
                        temperature,
                        pressure,
                        battery_potential,
                        (acceleration.0, acceleration.1, acceleration.2),
                    );

                    // Data Format 5 only
                    if let Some(measurement_seq) = sensor_values.measurement_sequence_number() {
                        sensor_data.measurement_sequence_number = Some(measurement_seq);
                    }
                    if let Some(movement_counter) = sensor_values.movement_counter() {
                        sensor_data.movement_counter = Some(movement_counter);
                    }
                    if let Some(tx_power) = sensor_values.tx_power_as_dbm() {
                        sensor_data.tx_power = Some(tx_power);
                    }
                    Some(Ok(sensor_data))
                }
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
