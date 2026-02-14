#![no_std]
#![no_main]

use crc::{Crc, CRC_8_NRSC_5};
use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::i2c::{Config as I2cConfig, I2c};
use embassy_stm32::rcc::{Hse, HseMode, Pll, PllMul, PllPreDiv, Sysclk};
use embassy_stm32::time::Hertz;
use embassy_time::Timer;
use embedded_aht20::{Aht20, DEFAULT_I2C_ADDRESS};
use panic_probe as _;

use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("=== AHT20 Sensor Reader ===");
    info!("Reading temperature and humidity data...");

    let mut config = embassy_stm32::Config::default();
    config.rcc.hse = Some(Hse {
        freq: Hertz(8_000_000),
        mode: HseMode::Oscillator,
    });
    config.rcc.pll_src = embassy_stm32::rcc::PllSource::HSE;
    config.rcc.pll = Some(Pll {
        prediv: PllPreDiv::DIV4,
        mul: PllMul::MUL168,
        divp: Some(embassy_stm32::rcc::PllPDiv::DIV2),
        divq: Some(embassy_stm32::rcc::PllQDiv::DIV7),
        divr: None,
    });
    config.rcc.ahb_pre = embassy_stm32::rcc::AHBPrescaler::DIV1;
    config.rcc.apb1_pre = embassy_stm32::rcc::APBPrescaler::DIV4;
    config.rcc.apb2_pre = embassy_stm32::rcc::APBPrescaler::DIV2;
    config.rcc.sys = Sysclk::PLL1_P;

    let p = embassy_stm32::init(config);

    info!("System initialized at 168MHz");

    let mut i2c_config = I2cConfig::default();
    i2c_config.frequency = Hertz(400_000);
    // i2c_config.sda_pullup = true;
    // i2c_config.scl_pullup = true;
    let i2c = I2c::new_blocking(p.I2C1, p.PB6, p.PB7, i2c_config);
    info!("I2C1 initialized at 100kHz with internal pull-ups");

    info!("Initializing AHT20 sensor...");
    let mut sensor = match Aht20::new(i2c, DEFAULT_I2C_ADDRESS, embassy_time::Delay) {
        Ok(s) => {
            info!("AHT20 sensor initialized successfully!");
            s
        }
        Err(_) => {
            error!("Failed to initialize AHT20 sensor!");
            error!("Check wiring and pull-up resistors.");
            loop {
                Timer::after_secs(1).await;
            }
        }
    };

    info!("Starting sensor readings...");
    info!("");

    loop {
        match sensor.measure_crc(|data: &[u8], crc: u8| {
            debug!("data: {}", data);
            debug!("crc: {}", crc);
            let crc_d = Crc::<u8>::new(&CRC_8_NRSC_5);
            let mut digest = crc_d.digest();
            digest.update(data);
            if digest.finalize() != crc {
                warn!("crc failed");
            }
            Ok(())
        }) {
            Ok(measurement) => {
                let temp = measurement.temperature.celsius();
                let humidity = measurement.relative_humidity;
                info!("Temperature: {} C | Humidity: {} %", temp, humidity);
            }
            Err(embedded_aht20::Error::I2c(_)) => {
                error!("Failed to read sensor data!I2c");
            }
            Err(embedded_aht20::Error::InvalidCrc) => {
                error!("Failed to read sensor data!InvalidCrc");
            }
            Err(embedded_aht20::Error::UnexpectedBusy) => {
                error!("Failed to read sensor data!UnexpectedBusy");
            }
        }

        Timer::after_millis(1000).await;
    }
}
