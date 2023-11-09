#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;

use fugit::RateExtU32;
use panic_probe as _;
use rp_pico::{
    hal::{
        self,
        clocks::init_clocks_and_plls,
        entry,
        gpio::{FunctionI2C, Pin, PullUp},
        pac,
        timer::Timer,
        watchdog::Watchdog,
        Sio, I2C,
    },
    Pins, XOSC_CRYSTAL_FREQ,
};

use keller_ld::{KellerLD, DEFAULT_ADDR};

#[entry]
fn main() -> ! {
    // -- Start init boilerplate
    info!("Start");
    // Soft-reset does not release the hardware spinlocks
    // Release them now to avoid a deadlock after debug or watchdog reset
    unsafe {
        hal::sio::spinlock_reset();
    }

    let mut pac = pac::Peripherals::take().unwrap();

    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    let pins = Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let clocks = init_clocks_and_plls(
        XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let timer = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    // -- End init boilerplate

    let sda_pin: Pin<_, FunctionI2C, PullUp> = pins.gpio0.reconfigure();
    let scl_pin: Pin<_, FunctionI2C, PullUp> = pins.gpio1.reconfigure();

    let i2c = I2C::i2c0(
        pac.I2C0,
        sda_pin,
        scl_pin,
        100.kHz(),
        &mut pac.RESETS,
        &clocks.system_clock,
    );

    let mut keller_ld = KellerLD::new(i2c, DEFAULT_ADDR, timer);
    let date = keller_ld.get_calibration().unwrap();
    info!("Calibration date: {}", date);

    loop {
        let measurement = keller_ld.read().unwrap();
        info!("Temperature: {} deg C", measurement.temperature);
        info!("Pressure: {} bar", measurement.pressure);
        info!("Depth: {} m", measurement.depth_underwater());
    }
}
