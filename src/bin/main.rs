#![no_std]
#![no_main]

use defmt::info;
use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock,
    delay::Delay,
    gpio::{Input, InputConfig, Level, Output, OutputConfig, Pull},
    main,
    spi::{
        Mode,
        master::{Address, Command, Config, DataMode, Spi},
    },
    time::Rate,
};
use esp_println as _;

// CRITICAL: This is required for the ESP32-S3 bootloader to recognize the app
esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    let delay = Delay::new();

    info!("Starting ST77916 QSPI Driver...");

    // 1. Setup Control Pins
    let mut rst = Output::new(peripherals.GPIO18, Level::High, OutputConfig::default());
    let mut _bl = Output::new(peripherals.GPIO17, Level::High, OutputConfig::default());

    loop {
        info!("Checking for life on TE pin (GPIO 16)...");
        // Toggle Reset (GPIO 18)
        rst.set_high();
        _bl.set_low();
        delay.delay_millis(2000);

        rst.set_low();
        _bl.set_high();
        delay.delay_millis(2000);
    }
}
