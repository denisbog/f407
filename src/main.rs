#![no_std]
#![no_main]

use core::any::Any;

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::i2c::{Config as I2cConfig, I2c};
use embassy_stm32::rcc::{self, Hse, HseMode, Pll, PllMul, PllPreDiv, Sysclk};
use embassy_stm32::time::Hertz;
use embassy_stm32_fsmc_display_interface::{FsmcLcd, Timing};
use embassy_time::Instant;
use embassy_time::{Delay, Timer};
use embedded_aht20::{Aht20, DEFAULT_I2C_ADDRESS};

use ili9341::{Ili9341, Orientation};
use panic_probe as _;

use {defmt_rtt as _, panic_probe as _};

// Display dimensions
const DISPLAY_WIDTH: u16 = 320;
const DISPLAY_HEIGHT: u16 = 240;

use embedded_graphics::{pixelcolor::Rgb565, prelude::*};

/// Hardware configuration for STM32F407 + ILI9341 using FSMC 8080 interface
///
/// Pin mapping for 16-bit parallel 8080 interface:
/// - PD7  -> FSMC_NE1   (Chip Select, CS)
/// - PD4  -> FSMC_NOE   (Read Enable, RD)
/// - PD5  -> FSMC_NWE   (Write Enable, WR)
/// - PD13 -> FSMC_A18   (Register Select/Data-Command, RS/DC)
/// - PD14, PD15, PD0, PD1 -> FSMC_D0-D3  (Data lines)
/// - PE7-PE15 -> FSMC_D4-D12             (Data lines)
/// - PD8-PD10 -> FSMC_D13-D15            (Data lines)
/// - PB0 -> RESET (GPIO output, consumed by Ili9341 driver)

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("=== STM32F407 ILI9341 LCD Display ===");
    info!("Starting application...");

    // Configure clocks for maximum performance
    // STM32F407 HSE = 8MHz, target SYSCLK = 168MHz
    let mut config = embassy_stm32::Config::default();
    config.rcc.hse = Some(Hse {
        freq: embassy_stm32::time::Hertz(8_000_000),
        mode: HseMode::Oscillator,
    });
    config.rcc.pll_src = rcc::PllSource::HSE;
    config.rcc.pll = Some(Pll {
        prediv: PllPreDiv::DIV4,        // 8MHz / 4 = 2MHz
        mul: PllMul::MUL168,            // 2MHz * 168 = 336MHz
        divp: Some(rcc::PllPDiv::DIV2), // 336MHz / 2 = 168MHz (SYSCLK)
        divq: Some(rcc::PllQDiv::DIV7), // 336MHz / 7 = 48MHz (USB)
        divr: None,
    });
    config.rcc.ahb_pre = rcc::AHBPrescaler::DIV1; // 168MHz
    config.rcc.apb1_pre = rcc::APBPrescaler::DIV4; // 42MHz
    config.rcc.apb2_pre = rcc::APBPrescaler::DIV2; // 84MHz
    config.rcc.sys = Sysclk::PLL1_P;

    // Initialize STM32 peripherals
    let p = embassy_stm32::init(config);

    info!("System initialized at 168MHz");

    // Configure reset pin as GPIO output
    let rst = Output::new(p.PB0, Level::High, Speed::High);

    // Create delay provider
    let mut delay = Delay;

    info!("Reset pin configured");

    // Initialize FMC peripheral for LCD
    info!("Initializing FSMC for 8080 interface...");

    let mut timing = Timing::default();
    timing.bus_turnaround = 1;
    timing.data = 4;
    timing.address_hold = 0;
    timing.address_setup = 0;

    // Create FSMC LCD interface with proper pin configuration
    // Uses Intel 8080 protocol via FSMC NOR/PSRAM Bank 1
    let lcd_interface = FsmcLcd::new(
        p.PD7,  // CS  (Chip Select / FSMC_NE1)
        p.PD4,  // RD  (Read Enable / FSMC_NOE)
        p.PD5,  // WR  (Write Enable / FSMC_NWE)
        p.PD13, // RS  (Register Select / FSMC_A18 - controls command/data)
        (
            p.PD14, p.PD15, p.PD0, p.PD1, // D0-D3
            p.PE7, p.PE8, p.PE9, p.PE10, // D4-D7
            p.PE11, p.PE12, p.PE13, p.PE14, // D8-D11
            p.PE15, p.PD8, p.PD9, p.PD10, // D12-D15
        ),
        &timing, // Read timing
        &timing, // Write timing
    );

    info!("FSMC interface created");

    // Initialize ILI9341 display driver
    info!("Initializing ILI9341 display driver...");

    let mut display = Ili9341::new(
        lcd_interface,
        rst,
        &mut delay,
        Orientation::Landscape,
        ili9341::DisplaySize240x320,
    )
    .expect("Failed to initialize display");

    info!("Display initialized successfully!");
    info!("Resolution: {}x{}", DISPLAY_WIDTH, DISPLAY_HEIGHT);

    // Initialize I2C1 for AHT20 sensor
    info!("Initializing I2C1 on PB6/PB7 for AHT20...");
    let mut i2c_config = I2cConfig::default();
    i2c_config.frequency = Hertz(100_000); // 100kHz standard mode
                                           // i2c_config.sda_pullup = true; // Enable internal pull-up on SDA
                                           // i2c_config.scl_pullup = true; // Enable internal pull-up on SCL
    let i2c = I2c::new_blocking(p.I2C1, p.PB6, p.PB7, i2c_config);
    info!("I2C1 initialized at 100kHz with internal pull-ups");

    // Initialize AHT20 sensor
    info!("Initializing AHT20 sensor...");
    let mut sensor =
        Aht20::new(i2c, DEFAULT_I2C_ADDRESS, Delay).expect("Failed to initialize AHT20 sensor");
    info!("AHT20 sensor initialized!");

    // The ili9341 driver provides low-level access
    // For drawing, you would implement a framebuffer or use the driver's methods
    // This example demonstrates the driver is working
    let colors = [
        ("RED", Rgb565::RED),
        ("GREEN", Rgb565::GREEN),
        ("BLUE", Rgb565::BLUE),
    ];
    info!("Starting display loop...");

    // Main loop - display colors and read sensor
    loop {
        // Read AHT20 sensor data
        match sensor.measure() {
            Ok(measurement) => {
                let temp = measurement.temperature.celcius();
                let humidity = measurement.relative_humidity;
                info!("AHT20: Temperature = {}°C, Humidity = {}%", temp, humidity);
            }
            Err(_e) => {
                error!("AHT20 read error");
            }
        }

        for (color_name, color) in colors.iter() {
            // Measure how long it takes to fill the screen
            let start = Instant::now();

            // Fill the entire screen with the current color
            display.clear(*color).unwrap();

            let duration = start.elapsed();

            // Log the color and time taken
            // 320x240 = 76,800 pixels at 16 bits per pixel = 153,600 bytes
            info!(
                "Filled screen with {} in {} ms ({} pixels, ~{} KB/s)",
                color_name,
                duration.as_millis(),
                320 * 240,
                if duration.as_millis() > 0 {
                    (153600 * 1000) / (duration.as_millis() as u64 * 1024)
                } else {
                    0
                }
            );
            // Wait 1 second before changing to the next color
            Timer::after_secs(1).await;
        }
    }
}
