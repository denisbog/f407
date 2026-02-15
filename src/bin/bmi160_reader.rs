#![no_std]
#![no_main]

use bmi160::{AccelerometerPowerMode, Bmi160, GyroscopePowerMode, SensorSelector, SlaveAddr};
use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::i2c::{Config as I2cConfig, I2c};
use embassy_stm32::rcc::{Hse, HseMode, Pll, PllMul, PllPreDiv, Sysclk};
use embassy_stm32::time::Hertz;
use embassy_stm32::{bind_interrupts, i2c, peripherals};
use embassy_time::Timer;
use panic_probe as _;

use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    I2C2_EV => i2c::EventInterruptHandler<peripherals::I2C2>;
    I2C2_ER => i2c::ErrorInterruptHandler<peripherals::I2C2>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("=== BMI160 Sensor Reader ===");
    info!("Reading accelerometer and gyroscope data...");

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

    // Initialize I2C2 in blocking mode for BMI160
    // I2C2: PB10 (SCL), PB11 (SDA)
    let mut i2c_config = I2cConfig::default();
    i2c_config.frequency = Hertz(100_000);
    let i2c = I2c::new_blocking(p.I2C2, p.PB10, p.PB11, i2c_config);
    info!("I2C2 initialized at 100kHz (blocking mode) on PB10/PB11");

    // Initialize BMI160 sensor (address 0x68 with SDO grounded)
    info!("Initializing BMI160 sensor at address 0x68...");
    let mut bmi160 = Bmi160::new_with_i2c(i2c, SlaveAddr::default());

    // Enable accelerometer and gyroscope
    match bmi160.set_accel_power_mode(AccelerometerPowerMode::Normal) {
        Ok(_) => info!("BMI160 accelerometer enabled"),
        Err(_) => {
            error!("Failed to enable BMI160 accelerometer");
            loop {
                Timer::after_secs(1).await;
            }
        }
    }

    match bmi160.set_gyro_power_mode(GyroscopePowerMode::Normal) {
        Ok(_) => info!("BMI160 gyroscope enabled"),
        Err(_) => {
            error!("Failed to enable BMI160 gyroscope");
            loop {
                Timer::after_secs(1).await;
            }
        }
    }

    info!("BMI160 sensor initialized successfully!");
    info!("");
    info!("Starting sensor readings...");
    info!("Accelerometer: ±2g range | Gyroscope: ±250°/s range");
    info!("");

    loop {
        match bmi160.data(SensorSelector::new().accel().gyro()) {
            Ok(data) => {
                if let (Some(accel), Some(gyro)) = (data.accel, data.gyro) {
                    // Convert raw values to human-readable units
                    // Default ranges: Accel ±2g, Gyro ±250°/s
                    // Raw values are 16-bit signed (-32768 to 32767)
                    let accel_g = (
                        (accel.x as f32) * 2.0 / 32768.0,
                        (accel.y as f32) * 2.0 / 32768.0,
                        (accel.z as f32) * 2.0 / 32768.0,
                    );
                    let gyro_dps = (
                        (gyro.x as f32) * 250.0 / 32768.0,
                        (gyro.y as f32) * 250.0 / 32768.0,
                        (gyro.z as f32) * 250.0 / 32768.0,
                    );

                    info!("=== Sensor Data ===");
                    info!("Accelerometer (g):");
                    info!("  X: {}g  Y: {}g  Z: {}g", accel_g.0, accel_g.1, accel_g.2);
                    info!("Gyroscope (dps):");
                    info!(
                        "  X: {}°  Y: {}°  Z: {}°",
                        gyro_dps.0, gyro_dps.1, gyro_dps.2
                    );
                    info!("");
                }
            }
            Err(_) => {
                error!("Failed to read BMI160 data");
            }
        }

        Timer::after_millis(500).await;
    }
}
