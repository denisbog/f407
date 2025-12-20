#![no_main]
#![no_std]

use cortex_m::asm;
use defmt_rtt as _;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::RgbColor;
use ili9341::Orientation;
// use lcd_ili9341::PixelFormat;
use panic_halt as _;
use stm32f4xx_hal::{
    dwt::DwtExt,
    gpio::GpioExt,
    pac::Peripherals,
    prelude::*,
    rcc::{Config, RccExt},
    timer::TimerExt,
};

use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();
    let mut rcc = dp.RCC.freeze(Config::hsi().sysclk(16.MHz()).pclk1(8.MHz()));

    let cp = cortex_m::peripheral::Peripherals::take().unwrap();
    let dwt = cp.DWT.constrain(cp.DCB, &rcc.clocks);
    let mut local_timer = dwt.delay();

    defmt::println!("led display");

    // let gpioa = dp.GPIOA.split(&mut rcc);
    let gpiod = dp.GPIOD.split(&mut rcc);
    let gpioe = dp.GPIOE.split(&mut rcc);

    let mut delay = dp.TIM1.delay(&mut rcc);
    let lcd = f407::LCD {
        wrx: gpiod.pd5.into_push_pull_output(),
        csx: gpiod.pd7.into_push_pull_output(),
        dcx: gpiod.pd13.into_push_pull_output(),
        rdx: gpiod.pd4.into_push_pull_output(),
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
        delay: &mut delay,
    };
    defmt::println!("lcd");
    // lcd.reset();
    let reset = gpioe.pe3.into_push_pull_output();
    let mut delay = dp.TIM3.delay_ms(&mut rcc);
    defmt::println!("controller");
    let mut controller = ili9341::Ili9341::new(
        lcd,
        reset,
        &mut delay,
        Orientation::Landscape,
        ili9341::DisplaySize240x320,
    )
    .unwrap();
    defmt::println!("loop");

    use embedded_graphics::{
        mono_font::{ascii::FONT_6X10, MonoTextStyle},
        pixelcolor::Rgb565,
        prelude::*,
        text::Text,
    };
    // Create a new character style
    let style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);
    loop {
        controller.clear(Rgb565::RED).unwrap();
        Text::new("Hello Rust!", Point::new(20, 30), style)
            .draw(&mut controller)
            .unwrap();
        local_timer.delay_ms(1000);
        controller.clear(Rgb565::BLUE).unwrap();
        Text::new("Hello Again Rust!", Point::new(20, 30), style)
            .draw(&mut controller)
            .unwrap();
        local_timer.delay_ms(1000);
    }
}
