#![forbid(unsafe_code)]
#![cfg_attr(not(test), no_std)]

use core::fmt;
use thiserror_no_std::Error;
use defmt::Format;
use embedded_hal::{delay::DelayNs, i2c::I2c};

pub type Celcius = f32;
pub type Bar = f32;
pub type Metre = f32;

pub const ATMOSPHERIC_PRESSURE: Bar = 1.01325;

pub const DEFAULT_ADDR: u8 = 0x40;
pub const REQUEST_MEASUREMENT: u8 = 0xAC;
pub const REQUEST_PRESSURE_MODE: u8 = 0x12;
pub const REQUEST_MIN_PRESSURE: u8 = 0x13;
pub const REQUEST_MAX_PRESSURE: u8 = 0x15;

const READ_DELAY: u32 = 10; // Milliseconds

pub struct KellerLD<I2C, D> {
    i2c: I2C,
    address: u8,
    delay: D,
    pub pressure_mode: Option<PressureMode>,
    pub max_pressure: Option<f32>,
    pub min_pressure: Option<f32>,
}

#[derive(Error, Debug, Format)]
pub enum KellerLDError {
    #[error("internal error")]
    UnexpectedValue,
    #[error("I2C communication problem")]
    Bus(embedded_hal::i2c::ErrorKind),
    #[error("must get calibration info before use")]
    Uncalibrated,
    #[error("wait for measurement")]
    Busy,
    #[error("sensor is not in 'normal mode'")]
    IncorrectMode,
    #[error("checksum mismatch")]
    ChecksumMismatch,
}

// Convert I²C errors
impl<E: embedded_hal::i2c::Error> From<E> for KellerLDError {
    fn from(e: E) -> Self {
        KellerLDError::Bus(e.kind())
    }
}
pub struct Measurement {
    pub temperature: Celcius,
    pub pressure: Bar,
}

impl Measurement {
    pub fn depth_underwater(&self) -> Metre {
        100.0 * (self.pressure - ATMOSPHERIC_PRESSURE) / 9.81
    }
}

#[derive(Debug, PartialEq, Format)]
pub struct Date {
    pub year: u16,
    pub month: u8,
    pub day: u8,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PressureMode {
    Vented,   // Zero at atmospheric pressure
    Sealed,   // Zero at 1.0 bar
    Absolute, // Zero at vacuum
}

impl PressureMode {
    pub fn offset(self) -> f32 {
        match self {
            PressureMode::Vented => ATMOSPHERIC_PRESSURE,
            PressureMode::Absolute => 0.0,
            PressureMode::Sealed => 1.0,
        }
    }
}

impl<I2C, D> KellerLD<I2C, D>
where
    I2C: I2c,
    D: DelayNs,
{
    pub fn new(i2c: I2C, address: u8, delay: D) -> Self {
        Self {
            i2c,
            address,
            delay,
            pressure_mode: None,
            max_pressure: None,
            min_pressure: None,
        }
    }

    pub fn get_calibration(&mut self) -> Result<Date, KellerLDError> {
        let date = self.get_pressure_mode()?;
        self.get_min_pressure()?;
        self.get_max_pressure()?;

        Ok(date)
    }

    pub fn read(&mut self) -> Result<Measurement, KellerLDError> {
        let mut data = [0; 5];
        self._read_write(&[REQUEST_MEASUREMENT], &mut data)?;

        let status = data[0];
        if status & 1 << 5 != 0 {
            return Err(KellerLDError::Busy);
        }
        if status & 0b11 << 3 != 0 {
            return Err(KellerLDError::IncorrectMode);
        }
        if status & 1 << 2 != 0 {
            return Err(KellerLDError::ChecksumMismatch);
        }

        let raw_pressure = u16::from_be_bytes(data[1..3].try_into().unwrap());
        let raw_temperature = u16::from_be_bytes(data[3..5].try_into().unwrap());
        Ok(Measurement {
            temperature: self._convert_temperature(raw_temperature),
            pressure: self._convert_pressure(raw_pressure)?,
        })
    }

    pub fn get_pressure_mode(&mut self) -> Result<Date, KellerLDError> {
        let mut data = [0; 3];
        self._read_write(&[REQUEST_PRESSURE_MODE], &mut data)?;
        let scaling_0 = u16::from_be_bytes(data[1..3].try_into().unwrap());

        self.pressure_mode = Some(match scaling_0 & 0b11 {
            0 => Ok(PressureMode::Vented),
            1 => Ok(PressureMode::Sealed),
            2 => Ok(PressureMode::Absolute),
            _ => Err(KellerLDError::UnexpectedValue),
        }?);

        Ok(Date {
            year: 2010 + (scaling_0 >> 11),
            month: ((scaling_0 & 0b1111 << 7) >> 7) as u8,
            day: (scaling_0 as u8 & 0b01111100) >> 2,
        })
    }

    pub fn get_min_pressure(&mut self) -> Result<(), KellerLDError> {
        let mut bytes = [0; 4];

        let mut data = [0; 3];
        self._read_write(&[REQUEST_MIN_PRESSURE], &mut data)?;
        bytes[0..2].copy_from_slice(&data[1..3]);

        self._read_write(&[REQUEST_MIN_PRESSURE + 1], &mut data)?;
        bytes[2..4].copy_from_slice(&data[1..3]);

        self.min_pressure = Some(f32::from_be_bytes(bytes));
        Ok(())
    }

    pub fn get_max_pressure(&mut self) -> Result<(), KellerLDError> {
        let mut bytes = [0; 4];
        let mut data = [0; 3];
        self._read_write(&[REQUEST_MAX_PRESSURE], &mut data)?;
        bytes[0..2].copy_from_slice(&data[1..3]);

        self._read_write(&[REQUEST_MAX_PRESSURE + 1], &mut data)?;
        bytes[2..4].copy_from_slice(&data[1..3]);

        self.max_pressure = Some(f32::from_be_bytes(bytes));
        Ok(())
    }

    fn _convert_pressure(&mut self, raw_pressure: u16) -> Result<Bar, KellerLDError> {
        if let (Some(mode), Some(min), Some(max)) =
            (self.pressure_mode, self.min_pressure, self.max_pressure)
        {
            Ok((raw_pressure as f32 / 32768.0 - 0.5) * (max - min) + min + mode.offset())
        } else {
            Err(KellerLDError::Uncalibrated)
        }
    }

    fn _convert_temperature(&mut self, raw_temperature: u16) -> Celcius {
        ((raw_temperature >> 4) - 24) as f32 * 0.05 - 50.0
    }

    fn _read_write(&mut self, write: &[u8], read: &mut [u8]) -> Result<(), KellerLDError> {
        self.i2c.write(self.address, write)?;
        self.delay.delay_ms(READ_DELAY);
        self.i2c.read(self.address, read)?;
        Ok(())
    }

    pub fn destroy(self) -> I2C {
        // Destroy driver instance, return I²C bus instance.
        self.i2c
    }
}

impl fmt::Display for Date {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "{:02}-{:02}-{}", self.day, self.month, self.year)
    }
}
