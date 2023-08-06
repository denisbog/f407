#![no_main]
#![no_std]

use panic_rtt_core as _;
#[rtic::app(device = stm32f4xx_hal::pac)]
mod app {
    use embedded_graphics::prelude::*;
    use rtt_target::rprintln;
    use stm32f4xx_hal::{
        prelude::*,
        spi::{NoMiso, Spi},
    };

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        rtt_target::rtt_init_print!();
        rprintln!("start");
        let dp = ctx.device;

        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.use_hse(25.MHz()).sysclk(80.MHz()).freeze();

        let gpioa = dp.GPIOA.split();

        let mut led = gpioa.pa6.into_push_pull_output();
        led.set_low();

        let clk = gpioa.pa5.into_alternate();

        let mosi = gpioa.pa7.into_alternate().internal_pull_up(true);

        let dc = gpioa.pa1.into_push_pull_output();
        let reset = gpioa.pa2.into_push_pull_output();
        let cs = gpioa.pa3.into_push_pull_output();
        let spi = Spi::new(
            dp.SPI1,
            (clk, NoMiso::new(), mosi),
            embedded_hal::spi::MODE_0,
            3.MHz(),
            &clocks,
        );

        let mut delay = dp.TIM1.delay_us(&clocks);

        rprintln!("create display");
        let mut lcd = ili9341::Ili9341::new(
            display_interface_spi::SPIInterface::new(spi, dc, cs),
            reset,
            &mut delay,
            ili9341::Orientation::Portrait,
            ili9341::DisplaySize240x320,
        )
        .unwrap();

        rprintln!("clear display");
        lcd.clear(embedded_graphics_core::pixelcolor::Rgb565::BLUE)
            .unwrap();

        let style = embedded_graphics::mono_font::MonoTextStyle::new(
            &embedded_graphics::mono_font::ascii::FONT_6X10,
            <embedded_graphics::pixelcolor::Rgb565 as embedded_graphics::prelude::RgbColor>::RED,
        );
        rprintln!("print string");
        embedded_graphics::text::Text::with_alignment(
            "some\ntext",
            embedded_graphics::prelude::Point::new(20, 30),
            style,
            embedded_graphics::text::Alignment::Center,
        )
        .draw(&mut lcd)
        .unwrap();
        led.set_high();
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
