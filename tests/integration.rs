use embedded_hal_mock::eh1 as embedded_hal;
use embedded_hal::delay::NoopDelay as DelayMock;
use embedded_hal::i2c::{Mock as I2cMock, Transaction};

use float_eq::assert_float_eq;

use keller_ld::{Date, KellerLD, DEFAULT_ADDR, PressureMode};

// See https://www.kelleramerica.com/file-cache/website_component/5e2f22709d8b1060a188c35e/manuals/1580321559752

#[test]
fn get_date_success() {
    let expectations = [
        Transaction::write(0x40, vec![0x12]),
        Transaction::read(0x40, vec![0x40, 0b00010101, 0b01110100]),
    ];
    let mock = I2cMock::new(&expectations);

    let mut keller_ld = KellerLD::new(mock, DEFAULT_ADDR, DelayMock);
    let date = keller_ld.get_pressure_mode().unwrap();

    assert_eq!(
        date,
        Date{
            year: 2012,
            month: 10,
            day: 29,
        }
    );

    keller_ld.destroy().done();
}

#[test]
fn get_pressure_mode_success() {
    let expectations = [
        Transaction::write(0x40, vec![0x12]),
        Transaction::read(0x40, vec![0x40, 0b00010101, 0b01110100]),
    ];
    let mock = I2cMock::new(&expectations);

    let mut keller_ld = KellerLD::new(mock, DEFAULT_ADDR, DelayMock);
    let _ = keller_ld.get_pressure_mode().unwrap();

    assert_eq!(
        keller_ld.pressure_mode,
        Some(PressureMode::Vented)
    );

    keller_ld.destroy().done();
}

#[test]
fn get_min_pressure_success() {
    let expectations = [
        Transaction::write(0x40, vec![0x13]),
        Transaction::read(0x40, vec![0x40, 0xBF, 0x80]),
        Transaction::write(0x40, vec![0x14]),
        Transaction::read(0x40, vec![0x40, 0x00, 0x00]),
    ];
    let mock = I2cMock::new(&expectations);

    let mut keller_ld = KellerLD::new(mock, DEFAULT_ADDR, DelayMock);
    let _ = keller_ld.get_min_pressure().unwrap();

    assert_eq!(
        keller_ld.min_pressure,
        Some(-1.0)
    );

    keller_ld.destroy().done();
}

#[test]
fn get_max_pressure_success() {
    let expectations = [
        Transaction::write(0x40, vec![0x15]),
        Transaction::read(0x40, vec![0x40, 0x41, 0x20]),
        Transaction::write(0x40, vec![0x16]),
        Transaction::read(0x40, vec![0x40, 0x00, 0x00]),
    ];
    let mock = I2cMock::new(&expectations);

    let mut keller_ld = KellerLD::new(mock, DEFAULT_ADDR, DelayMock);
    let _ = keller_ld.get_max_pressure().unwrap();

    assert_eq!(
        keller_ld.max_pressure,
        Some(10.0)
    );

    keller_ld.destroy().done();
}

#[test]
fn get_calibration_success() {
    let expectations = [
        Transaction::write(0x40, vec![0x12]),
        Transaction::read(0x40, vec![0x40, 0b00010101, 0b01110100]),
        Transaction::write(0x40, vec![0x13]),
        Transaction::read(0x40, vec![0x40, 0xBF, 0x80]),
        Transaction::write(0x40, vec![0x14]),
        Transaction::read(0x40, vec![0x40, 0x00, 0x00]),        
        Transaction::write(0x40, vec![0x15]),
        Transaction::read(0x40, vec![0x40, 0x41, 0x20]),
        Transaction::write(0x40, vec![0x16]),
        Transaction::read(0x40, vec![0x40, 0x00, 0x00]),
    ];
    let mock = I2cMock::new(&expectations);

    let mut keller_ld = KellerLD::new(mock, DEFAULT_ADDR, DelayMock);
    let _ = keller_ld.get_calibration().unwrap();

    keller_ld.destroy().done();
}

#[test]
fn read_pressure_pr_7ld_success() {
    let expectations = [
        Transaction::write(0x40, vec![0xAC]),
        Transaction::read(0x40, vec![0x40, 0x4E, 0x20, 0x5D, 0xD1]),
    ];
    let mock = I2cMock::new(&expectations);

    let mut keller_ld = KellerLD::new(mock, DEFAULT_ADDR, DelayMock);
    keller_ld.pressure_mode = Some(PressureMode::Absolute);
    keller_ld.min_pressure = Some(-1.0);
    keller_ld.max_pressure = Some(10.0);

    let measurement = keller_ld.read().unwrap();

    assert_float_eq!(
        measurement.pressure,
        0.213867,
        abs <= 1.0E-6
    );

    keller_ld.destroy().done();
}

#[test]
fn read_pressure_pa_4ld_success() {
    let expectations = [
        Transaction::write(0x40, vec![0xAC]),
        Transaction::read(0x40, vec![0x40, 0x4E, 0x20, 0x5D, 0xD1]),
    ];
    let mock = I2cMock::new(&expectations);

    let mut keller_ld = KellerLD::new(mock, DEFAULT_ADDR, DelayMock);
    keller_ld.pressure_mode = Some(PressureMode::Sealed);
    keller_ld.min_pressure = Some(0.0);
    keller_ld.max_pressure = Some(30.0);

    let measurement = keller_ld.read().unwrap();

    assert_float_eq!(
        measurement.pressure,
        4.31055,
        abs <= 1.0E-5
    );

    keller_ld.destroy().done();
}

#[test]
fn read_pressure_paa_9ld_success() {
    let expectations = [
        Transaction::write(0x40, vec![0xAC]),
        Transaction::read(0x40, vec![0x40, 0x4E, 0x20, 0x5D, 0xD1]),
    ];
    let mock = I2cMock::new(&expectations);

    let mut keller_ld = KellerLD::new(mock, DEFAULT_ADDR, DelayMock);
    keller_ld.pressure_mode = Some(PressureMode::Absolute);
    keller_ld.min_pressure = Some(0.0);
    keller_ld.max_pressure = Some(3.0);

    let measurement = keller_ld.read().unwrap();

    assert_float_eq!(
        measurement.pressure,
        0.331055,
        abs <= 1.0E-6
    );

    keller_ld.destroy().done();
}

#[test]
fn read_temperature_success() {
    let expectations = [
        Transaction::write(0x40, vec![0xAC]),
        Transaction::read(0x40, vec![0x40, 0x4E, 0x20, 0x5D, 0xD1]),
    ];
    let mock = I2cMock::new(&expectations);

    let mut keller_ld = KellerLD::new(mock, DEFAULT_ADDR, DelayMock);
    keller_ld.pressure_mode = Some(PressureMode::Absolute);
    keller_ld.min_pressure = Some(-1.0);
    keller_ld.max_pressure = Some(10.0);

    let measurement = keller_ld.read().unwrap();

    assert_float_eq!(
        measurement.temperature,
        23.85,
        abs <= 1.0E-2
    );

    keller_ld.destroy().done();
}