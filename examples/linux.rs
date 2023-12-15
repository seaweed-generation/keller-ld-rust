use embedded_hal::delay::DelayNs;
use linux_embedded_hal::{Delay, I2cdev};
use keller_ld::{KellerLD, DEFAULT_ADDR};

fn main() {
    let i2c = I2cdev::new("/dev/i2c-1").unwrap();
    let mut keller_ld = KellerLD::new(i2c, DEFAULT_ADDR, Delay);
    let date = keller_ld.get_calibration().unwrap();

    println!("Calibration date: {}", date);

    loop {
        let measurement = keller_ld.read().unwrap();
        println!("Temperature: {} deg C", measurement.temperature);
        println!("Pressure: {} bar", measurement.pressure);
        println!("Depth: {} m", measurement.depth_underwater());

        Delay.delay_ms(1000);
    }
}