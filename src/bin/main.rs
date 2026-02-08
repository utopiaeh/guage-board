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

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    let delay = Delay::new();

    info!("Starting ST77916 Safe-Mode Driver (Mode 2, 2MHz)...");

    // 1. Control Pins
    let mut rst = Output::new(peripherals.GPIO18, Level::High, OutputConfig::default());
    let _bl = Output::new(peripherals.GPIO17, Level::High, OutputConfig::default());
    let mut cs = Output::new(peripherals.GPIO10, Level::High, OutputConfig::default());

    let te_pin = Input::new(
        peripherals.GPIO16,
        InputConfig::default().with_pull(Pull::None),
    );

    // 2. Hardware Reset (Crucial for clearing previous failed states)
    rst.set_high();
    delay.delay_millis(50);
    rst.set_low();
    delay.delay_millis(150);
    rst.set_high();
    delay.delay_millis(250);

    // 3. SPI Configuration (Mode 2 + 2MHz for command reliability)
    let spi_config = Config::default()
        .with_frequency(Rate::from_mhz(10))
        .with_mode(Mode::_2);

    let mut spi = Spi::new(peripherals.SPI2, spi_config)
        .expect("Failed to init SPI")
        .with_sck(peripherals.GPIO12)
        .with_mosi(peripherals.GPIO11)
        .with_miso(peripherals.GPIO13)
        .with_sio2(peripherals.GPIO14)
        .with_sio3(peripherals.GPIO9);

    // 4. Macro for Commands (Standard 8-bit Phase)
    macro_rules! send_cmd {
        ($cmd:expr, $data:expr) => {
            cs.set_low();
            let _ = spi.half_duplex_write(
                DataMode::Single,
                Command::_8Bit($cmd as u16, DataMode::Single),
                Address::_32Bit(($cmd as u32) << 24, DataMode::Single),
                0,
                $data,
            );
            cs.set_high();
            delay.delay_millis(5);
        };
    }

    // 5. Wake & Advanced Calibration Sequence
    info!("Waking and Calibrating ST77916...");
    send_cmd!(0x11, &[]); // Sleep Out
    delay.delay_millis(150);

    // UNLOCK Command List 2
    send_cmd!(0xF0, &[0xC3]);
    send_cmd!(0xF0, &[0x96]);

    // --- CALIBRATION DATA (The "Magic" to stop the lines) ---
    send_cmd!(0xC0, &[0x80, 0x20]); // Power Control 1
    send_cmd!(0xC1, &[0x02]); // Power Control 2 (Pump Frequency)
    send_cmd!(0xE2, &[0x03, 0x00, 0x00, 0x03]); // Source Timing
    send_cmd!(0xE5, &[0x01]); // Gate Timing
    send_cmd!(0x3B, &[0x03, 0x03, 0x03, 0x03]); // Frame Rate (60Hz)

    // Interface Setup
    send_cmd!(0x3A, &[0x55]); // 16-bit RGB565
    send_cmd!(0x36, &[0x00]); // MADCTL (Direction)

    // Brightness (Ensures pixels have enough 'punch')
    send_cmd!(0x51, &[0xFF]);
    send_cmd!(0x53, &[0x24]);

    send_cmd!(0x21, &[]); // Inversion ON
    send_cmd!(0x29, &[]); // Display ON
    delay.delay_millis(50);

    // Attempt standard Power-up
    send_cmd!(0x21, &[]); // Inversion ON
    send_cmd!(0x29, &[]); // Display ON
    delay.delay_millis(100);

    // 6. Fill Screen with RED (Quad Mode)
    info!("Filling Screen with RED using Quad Data...");
    send_cmd!(0x2A, &[0x00, 0x00, 0x01, 0x67]);
    send_cmd!(0x2B, &[0x00, 0x00, 0x01, 0x67]);
    send_cmd!(0x2C, &[]); // Start RAM Write

    let red_pixel = [0xF8u8, 0x00u8];
    let line_buffer = [red_pixel; 360];
    let raw_line: &[u8] =
        unsafe { core::slice::from_raw_parts(line_buffer.as_ptr() as *const u8, 720) };

    for _ in 0..360 {
        cs.set_low();
        // CRITICAL CHANGE: Use DataMode::Quad for the pixel payload
        let _ = spi.half_duplex_write(
            DataMode::Quad, // Switch to 4-lane data here
            Command::None,
            Address::None,
            0,
            raw_line,
        );
        cs.set_high();
    }

    info!("Monitoring TE Pin Activity...");
    loop {
        let mut toggles = 0;
        let mut last_state = te_pin.is_high();
        for _ in 0..100 {
            let current_state = te_pin.is_high();
            if current_state != last_state {
                toggles += 1;
                last_state = current_state;
            }
            delay.delay_millis(10);
        }

        if toggles > 0 {
            info!("SUCCESS: TE Active ({} toggles).", toggles);
        } else {
            info!("TE STATIC: Trying Wake again...");
            send_cmd!(0x11, &[]);
            delay.delay_millis(200);
            send_cmd!(0x35, &[0x00]);
        }
    }
}
