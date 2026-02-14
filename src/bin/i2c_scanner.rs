#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::i2c::{Config as I2cConfig, I2c};
use embassy_stm32::rcc::{self, Hse, HseMode, Pll, PllMul, PllPreDiv, Sysclk};
use embassy_stm32::time::Hertz;
use embassy_time::Timer;
use embedded_hal::i2c::I2c as I2cTrait;
use panic_probe as _;

use {defmt_rtt as _, panic_probe as _};

const I2C_ADDRESS_RANGE_START: u8 = 0x08;
const I2C_ADDRESS_RANGE_END: u8 = 0x77;

fn scan_i2c_bus(
    i2c: &mut I2c<'_, embassy_stm32::mode::Blocking, embassy_stm32::i2c::Master>,
) -> [bool; 128] {
    let mut devices = [false; 128];

    for addr in I2C_ADDRESS_RANGE_START..=I2C_ADDRESS_RANGE_END {
        match I2cTrait::write(i2c, addr, &[]) {
            Ok(()) => {
                devices[addr as usize] = true;
            }
            Err(_) => {}
        }
    }
    devices
}

fn print_i2c_table(devices: &[bool; 128]) {
    info!("I2C Bus Scan Results:");
    info!("====================");
    info!("     0  1  2  3  4  5  6  7  8  9  A  B  C  D  E  F");

    for row in 0..8u8 {
        let start_addr = row * 16;

        for col in 0..16u8 {
            let addr = (start_addr + col) as usize;
            if devices[addr] {
                info!(
                    "0x{:02X}: Device found at 0x{:02X} ({})",
                    start_addr, addr, addr
                );
                break;
            }
        }
    }

    info!("====================");

    let mut found_count = 0;
    for addr in I2C_ADDRESS_RANGE_START..=I2C_ADDRESS_RANGE_END {
        if devices[addr as usize] {
            found_count += 1;
        }
    }

    if found_count == 0 {
        warn!("No I2C devices found!");
    } else {
        info!("Found {} device(s):", found_count);
        for addr in I2C_ADDRESS_RANGE_START..=I2C_ADDRESS_RANGE_END {
            if devices[addr as usize] {
                info!("  - 0x{:02X} ({})", addr, addr);
            }
        }
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("=== I2C Device Scanner ===");
    info!("Scanning for I2C devices on PB6/PB7...");

    let mut config = embassy_stm32::Config::default();
    config.rcc.hse = Some(Hse {
        freq: embassy_stm32::time::Hertz(8_000_000),
        mode: HseMode::Oscillator,
    });
    config.rcc.pll_src = rcc::PllSource::HSE;
    config.rcc.pll = Some(Pll {
        prediv: PllPreDiv::DIV4,
        mul: PllMul::MUL168,
        divp: Some(rcc::PllPDiv::DIV2),
        divq: Some(rcc::PllQDiv::DIV7),
        divr: None,
    });
    config.rcc.ahb_pre = rcc::AHBPrescaler::DIV1;
    config.rcc.apb1_pre = rcc::APBPrescaler::DIV4;
    config.rcc.apb2_pre = rcc::APBPrescaler::DIV2;
    config.rcc.sys = Sysclk::PLL1_P;

    let p = embassy_stm32::init(config);

    info!("System initialized at 168MHz");

    let mut i2c_config = I2cConfig::default();
    i2c_config.frequency = Hertz(100_000);
    // i2c_config.sda_pullup = true;
    // i2c_config.scl_pullup = true;
    let mut i2c = I2c::new_blocking(p.I2C1, p.PB6, p.PB7, i2c_config);
    info!("I2C1 initialized at 100kHz with internal pull-ups");

    loop {
        info!("");
        info!("Starting scan...");

        let devices = scan_i2c_bus(&mut i2c);
        print_i2c_table(&devices);

        info!("");
        info!("Waiting 2 seconds before next scan...");
        Timer::after_secs(2).await;
    }
}
