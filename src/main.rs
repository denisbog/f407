#![no_main]
#![no_std]

use panic_halt as _;

#[rtic::app(device = stm32f4xx_hal::pac,  peripherals = true)]
mod app {
    use cortex_m::singleton;
    use stm32f4xx_hal::{
        dma::{config::DmaConfig, StreamsTuple},
        prelude::*,
        spi::{NoMiso, Spi},
    };

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        let dp = ctx.device;

        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.use_hse(25.MHz()).sysclk(80.MHz()).freeze();

        let gpioa = dp.GPIOA.split();

        let mut led = gpioa.pa6.into_push_pull_output();
        led.set_low();

        let clk = gpioa
            .pa5
            .into_alternate()
            .speed(stm32f4xx_hal::gpio::Speed::VeryHigh)
            .internal_pull_up(true);

        let mosi = gpioa
            .pa7
            .into_alternate()
            .speed(stm32f4xx_hal::gpio::Speed::VeryHigh);

        let spi = Spi::new(
            dp.SPI1,
            (clk, NoMiso::new(), mosi),
            embedded_hal::spi::MODE_0,
            3.MHz(),
            &clocks,
        );

        let tx = spi.use_dma().tx();
        let streams = StreamsTuple::new(dp.DMA2);
        let stream = streams.3;

        let buf = singleton!(: [u8; 100] = [1;100]).unwrap();

        for (i, b) in buf.iter_mut().enumerate() {
            *b = i as u8;
        }

        let mut transfer = stm32f4xx_hal::dma::Transfer::init_memory_to_peripheral(
            stream,
            tx,
            buf,
            None,
            DmaConfig::default()
                .memory_increment(true)
                .fifo_enable(true)
                .fifo_error_interrupt(true)
                .transfer_complete_interrupt(true),
        );

        transfer.start(|_tx| {
            led.set_high();
        });

        (Shared {}, Local {}, init::Monotonics())
    }

    #[idle(local = [])]
    fn idle(_cx: idle::Context) -> ! {
        loop {
            cortex_m::asm::nop();
        }
    }
}
