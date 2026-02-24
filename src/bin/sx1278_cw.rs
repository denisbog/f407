#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::mode::Blocking;
use embassy_stm32::spi::{Config, Spi};
use embassy_stm32::time::Hertz;
use panic_probe as _;

use {defmt_rtt as _, panic_probe as _};

const SX1278_FREQ: u32 = 433_200_000;

const REG_OP_MODE: u8 = 0x01;
const REG_FRF_MSB: u8 = 0x06;
const REG_FRF_MID: u8 = 0x07;
const REG_FRF_LSB: u8 = 0x08;
const REG_PA_CONFIG: u8 = 0x09;
const REG_PA_RAMP: u8 = 0x0A;
const REG_LNA: u8 = 0x0C;
const REG_FIFO_TX_BASE_ADDR: u8 = 0x0E;
const REG_FIFO_ADDR_PTR: u8 = 0x0D;
const REG_IRQ_FLAGS_MASK: u8 = 0x11;
const REG_IRQ_FLAGS: u8 = 0x12;
const REG_DIO_MAPPING_1: u8 = 0x40;
const REG_PA_DAC: u8 = 0x4D;

const MODE_LONG_RANGE: u8 = 0x80;
const MODE_SLEEP: u8 = 0x00;
const MODE_STDBY: u8 = 0x01;
const MODE_TX: u8 = 0x03;

const PA_DAC_HIGH: u8 = 0x87;

fn spi_write(spi: &mut Spi<'_, Blocking>, nss: &mut Output<'_>, reg: u8, value: u8) {
    nss.set_low();
    let buffer = [reg | 0x80, value];
    spi.blocking_write(&buffer).ok();
    nss.set_high();
}

fn spi_read(spi: &mut Spi<'_, Blocking>, nss: &mut Output<'_>, reg: u8) -> u8 {
    nss.set_low();
    let mut buffer = [reg, 0x00];
    spi.blocking_transfer_in_place(&mut buffer).ok();
    nss.set_high();
    buffer[1]
}

fn set_frequency(spi: &mut Spi<'_, Blocking>, nss: &mut Output<'_>, freq: u32) {
    let frf = (freq as u64) << 19;
    let frf = frf / 32_000_000;
    let msb = ((frf >> 16) & 0xFF) as u8;
    let mid = ((frf >> 8) & 0xFF) as u8;
    let lsb = (frf & 0xFF) as u8;
    spi_write(spi, nss, REG_FRF_MSB, msb);
    spi_write(spi, nss, REG_FRF_MID, mid);
    spi_write(spi, nss, REG_FRF_LSB, lsb);
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("=== SX1278 Continuous Wave Generator ===");

    let config = embassy_stm32::Config::default();
    let p = embassy_stm32::init(config);

    info!("Configuring SPI1 for SX1278...");

    let mut spi_config = Config::default();
    spi_config.frequency = Hertz(1_000_000);

    let mut spi = Spi::new_blocking(p.SPI1, p.PA5, p.PA7, p.PA6, spi_config);
    info!("SPI1 initialized at 1MHz on PA5/PA7/PA6");

    let mut nss = Output::new(p.PA4, Level::High, Speed::High);
    let mut reset = Output::new(p.PC4, Level::High, Speed::High);

    info!("NSS pin: PA4, Reset pin: PC4");

    nss.set_high();

    reset.set_low();
    cortex_m::asm::delay(10_000);
    reset.set_high();
    cortex_m::asm::delay(10_000);

    info!("SX1278 reset complete");

    let version = spi_read(&mut spi, &mut nss, 0x42);
    info!("SX1278 version: {:#04x}", version);

    if version != 0x12 {
        error!("Unexpected SX1278 version! Expected 0x12");
    }

    spi_write(
        &mut spi,
        &mut nss,
        REG_OP_MODE,
        MODE_LONG_RANGE | MODE_SLEEP,
    );
    cortex_m::asm::delay(10_000);

    let op_mode = spi_read(&mut spi, &mut nss, REG_OP_MODE);
    info!("Op mode after sleep: {:#04x}", op_mode);

    spi_write(&mut spi, &mut nss, 0x31, 0x18);
    spi_write(&mut spi, &mut nss, 0x2E, 0x0);
    spi_write(&mut spi, &mut nss, 0x3E, 0x0);

    spi_write(&mut spi, &mut nss, REG_FIFO_TX_BASE_ADDR, 0);
    spi_write(&mut spi, &mut nss, REG_FIFO_ADDR_PTR, 0);

    spi_write(&mut spi, &mut nss, REG_LNA, 0x23);

    spi_write(&mut spi, &mut nss, REG_PA_RAMP, 0x09);

    spi_write(&mut spi, &mut nss, REG_PA_CONFIG, 0xFF);

    set_frequency(&mut spi, &mut nss, SX1278_FREQ);
    info!("Frequency set to {} Hz", SX1278_FREQ);

    spi_write(&mut spi, &mut nss, REG_PA_DAC, PA_DAC_HIGH);
    info!("PA_DAC enabled for +20 dBm");

    spi_write(
        &mut spi,
        &mut nss,
        REG_OP_MODE,
        MODE_LONG_RANGE | MODE_STDBY,
    );
    cortex_m::asm::delay(10_000);

    spi_write(&mut spi, &mut nss, REG_DIO_MAPPING_1, 0x40);
    spi_write(&mut spi, &mut nss, REG_IRQ_FLAGS_MASK, 0xF7);
    spi_write(&mut spi, &mut nss, REG_IRQ_FLAGS, 0xFF);

    info!("Entering continuous TX mode...");
    spi_write(&mut spi, &mut nss, REG_OP_MODE, MODE_LONG_RANGE | MODE_TX);

    info!(
        "Continuous wave TX started at {} MHz",
        SX1278_FREQ / 1_000_000
    );

    loop {
        cortex_m::asm::wfi();
    }
}
