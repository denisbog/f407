#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;
use stm32f4xx_hal::{
    pac,
    prelude::*,
    spi::{Mode, Phase, Polarity, Spi},
    timer::SysDelay,
};

use embedded_hal_bus::spi::ExclusiveDevice;
use sx1262::commands::{
    operational::{GetIrqStatus, SetStandby, StandbyConfig},
    rf::{PacketType, SetPacketType, SetRfFrequency},
};
use sx1262::Device;

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let cp = pac::CorePeripherals::take().unwrap();

    // 1. Clock Configuration (v0.23.0 style)
    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr().free.sysclk(48.MHz()).freeze();

    // Create a delay provider (used by the SPI bus wrapper)
    let mut delay = cp.SYST.delay(&clocks);

    let gpioa = dp.GPIOA.split(&mut rcc);
    let gpiob = dp.GPIOB.split(&mut rcc);

    // 2. SPI Pins (wrapped in Some() for v0.23.0)
    let sck = gpioa.pa5.into_alternate();
    let miso = gpioa.pa6.into_alternate();
    let mosi = gpioa.pa7.into_alternate();
    let mut nss = gpiob.pb6.into_push_pull_output(); // Chip Select

    // Ensure NSS starts high (inactive)
    nss.set_high();

    let spi_bus = Spi::new(
        dp.SPI1,
        (Some(sck), Some(miso), Some(mosi)),
        Mode {
            polarity: Polarity::IdleLow,
            phase: Phase::CaptureOnFirstTransition,
        },
        8.MHz(),
        &clocks,
    );

    // 3. The SPI Device Wrapper
    // This handles the NSS pin automatically during transactions
    let spi_device = ExclusiveDevice::new(spi_bus, nss, delay);

    // 4. Initialize SX1262
    let mut lora = Device::new(spi_device);

    // Enter Standby to configure
    lora.execute_command(SetStandby {
        config: StandbyConfig::Rc,
    })
    .unwrap();

    // Set to LoRa
    lora.execute_command(SetPacketType {
        packet_type: PacketType::LoRa,
    })
    .unwrap();

    // Set Frequency to 868MHz
    lora.execute_command(SetRfFrequency {
        rf_frequency: 0x36400000,
    })
    .unwrap();

    loop {
        // 5. Checking IRQ Status (The "Polled" way or inside the Interrupt)
        // This is how you read back the result of the GetIrqStatus command
        let response = lora.execute_command(GetIrqStatus).unwrap();

        // Check if TxDone bit (bit 0) is set
        if response.irq_status & 0x0001 != 0 {
            // Packet sent!
            // You would then clear the IRQ status on the radio here
        }
    }
}
