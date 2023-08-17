#![no_main]
#![no_std]

use panic_rtt_core as _;
#[rtic::app(device = stm32f4xx_hal::pac)]
mod app {
    use lcd_ili9341::PixelFormat;
    use rtt_target::rprintln;
    use stm32f4xx_hal::prelude::*;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        // rtt_target::rtt_init_print!();
        rprintln!("start");
        let dp = ctx.device;

        let rcc = dp.RCC.constrain();
        let clocks = rcc
            .cfgr
            //.use_hse(8.MHz()).sysclk(8.MHz())
            .freeze();
        let gpioa = dp.GPIOA.split();
        let gpiob = dp.GPIOB.split();
        let gpiod = dp.GPIOD.split();
        let gpioe = dp.GPIOE.split();
        let mut delay = dp.TIM1.delay_us(&clocks);

        let lcd = f407::LCD {
            csx: gpiob.pb12.into_push_pull_output(),
            dcx: gpiod.pd7.into_push_pull_output(),
            wrx: gpiod.pd4.into_push_pull_output(),
            rdx: gpiod.pd5.into_push_pull_output(),
            d0: gpiod.pd14.into_push_pull_output(),
            d1: gpiod.pd15.into_push_pull_output(),
            d2: gpiod.pd0.into_push_pull_output(),
            d3: gpiod.pd1.into_push_pull_output(),
            d4: gpioe.pe7.into_push_pull_output(),
            d5: gpioe.pe8.into_push_pull_output(),
            d6: gpioe.pe9.into_push_pull_output(),
            d7: gpioe.pe10.into_push_pull_output(),
            d8: gpioe.pe11.into_push_pull_output(),
            d9: gpioe.pe12.into_push_pull_output(),
            d10: gpioe.pe13.into_push_pull_output(),
            d11: gpioe.pe14.into_push_pull_output(),
            d12: gpioe.pe15.into_push_pull_output(),
            d13: gpiod.pd8.into_push_pull_output(),
            d14: gpiod.pd9.into_push_pull_output(),
            d15: gpiod.pd10.into_push_pull_output(),
            resx: gpioe.pe4.into_push_pull_output(),
            delay: &mut dp.TIM2.delay_us(&clocks),
        };

        let mut controller = lcd_ili9341::Controller::new(lcd);
        // //reset start
        let mut reset = gpioe.pe3.into_push_pull_output();

        reset.set_low();
        delay.delay(10.micros());
        reset.set_high();
        delay.delay(5.millis());
        // //reset end

        let mut led = gpioa.pa6.into_push_pull_output();
        led.set_high();

        // delay.delay(5.secs());
        delay.delay(5.millis());
        controller.software_reset();
        delay.delay(120.millis());

        // controller.sleep_out();
        // delay.delay(100.millis());

        // controller.display(false);
        // delay.delay(5.millis());
        //
        // delay.delay(5.secs());

        controller.display(true);
        delay.delay(5.millis());

        // delay.delay(5.secs());

        controller.pixel_format_set(PixelFormat::bit16());
        delay.delay(5.millis());
        controller.sleep_out();
        delay.delay(5.millis());
        controller.column_address_set(0x0000u16, 0x0900u16);
        delay.delay(5.millis());
        controller.page_address_set(0x0000u16, 0x0009u16);

        delay.delay(5.millis());
        controller.memory_write_start();
        delay.delay(5.millis());
        controller.write_memory(core::iter::repeat(0b1111100000000000).take(240 * 5));
        delay.delay(5.millis());
        led.set_low();
        rprintln!("done");
        (Shared {}, Local {}, init::Monotonics())
    }

    #[idle(local = [])]
    fn idle(_cx: idle::Context) -> ! {
        loop {
            cortex_m::asm::nop();
        }
    }
}
