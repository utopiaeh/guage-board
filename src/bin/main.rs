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

    info!("Starting ST77916 on JSON Pinout (GPIO 9-14)...");

    // 1. Setup Control Pins (Reset 18, Backlight 17, CS 10)
    let mut rst = Output::new(peripherals.GPIO18, Level::High, OutputConfig::default());
    let _bl = Output::new(peripherals.GPIO17, Level::High, OutputConfig::default());
    let mut cs = Output::new(peripherals.GPIO10, Level::High, OutputConfig::default());

    let te_pin = Input::new(
        peripherals.GPIO16,
        InputConfig::default().with_pull(Pull::None),
    );

    // 2. Hardware Reset Sequence (ST77916 is very sensitive to this)
    rst.set_high();
    delay.delay_millis(50);
    rst.set_low();
    delay.delay_millis(150); // Hold reset
    rst.set_high();
    delay.delay_millis(200); // Wait for boot

    // 3. SPI Configuration (Slowed to 1MHz for absolute stability)
    let spi_config = Config::default()
        .with_frequency(Rate::from_mhz(10))
        .with_mode(Mode::_2);

    let mut spi = Spi::new(peripherals.SPI2, spi_config)
        .expect("Failed to init SPI")
        .with_sck(peripherals.GPIO12) // SCL
        .with_mosi(peripherals.GPIO11) // SDA (IO0)
        .with_miso(peripherals.GPIO13) // IO1
        .with_sio2(peripherals.GPIO14) // IO2
        .with_sio3(peripherals.GPIO9); // IO3

    // 4. Helper for Manual CS writes
    let mut send_cmd = |cmd: u8, data: &[u8]| {
        cs.set_low();
        delay.delay_micros(10);

        let _ = spi.half_duplex_write(
            DataMode::Single,
            Command::None,
            // We send the command as a 32-bit Address.
            // Some displays expect the command in the first byte, some in the last.
            // This version puts it in the first byte.
            Address::_32Bit((cmd as u32) << 24, DataMode::Single),
            0,
            data,
        );

        delay.delay_micros(10);
        cs.set_high();
        delay.delay_millis(5);
    };

    // 5. Wake and Unlock
    info!("Sending Wake (0x11)...");
    send_cmd(0x11, &[]);
    delay.delay_millis(150);

    send_cmd(0x3A, &[0x55]); // Set 16-bit RGB565
    send_cmd(0x35, &[0x00]); // Force TE ON
    info!("Unlocking Manufacturer Registers...");
    send_cmd(0xF0, &[0xC3]);
    send_cmd(0xF0, &[0x96]);

    // Mandatory settings to start the internal clock
    send_cmd(0x35, &[0x00]); // TE ON (This makes GPIO 16 move!)
    send_cmd(0x3A, &[0x55]); // 16-bit
    send_cmd(0x21, &[]); // Inversion
    send_cmd(0x29, &[]); // Display ON

    info!("Entering Monitor Loop...");
    loop {
        // LIFE CHECK on GPIO 16
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
            info!("SUCCESS: TE Pin is toggling! Display is communicating.");
        } else {
            info!("FAILURE: TE pin (GPIO 16) is static. Display is not awake.");
        }

        // Toggle Display for visibility
        send_cmd(0x28, &[]); // OFF
        delay.delay_millis(500);
        send_cmd(0x29, &[]); // ON
        delay.delay_millis(500);
    }
}
