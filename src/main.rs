#![no_main]
#![no_std]

use core::u16;

use cortex_m::asm::delay;
use defmt_rtt as _;
use lcd_ili9341::{MemoryAccessControl, PixelFormat};
use panic_halt as _;
use stm32f4xx_hal::{
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
    defmt::println!("led display");

    let gpioa = dp.GPIOA.split(&mut rcc);
    let gpiod = dp.GPIOD.split(&mut rcc);
    let gpioe = dp.GPIOE.split(&mut rcc);

    let mut delay = dp.TIM1.delay(&mut rcc);

    let mut lcd = f407::LCD {
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
    // lcd.reset();
    let mut delay = dp.TIM2.delay_us(&mut rcc);
    // //TODO: check what reset is doing
    // let mut reset = gpioe.pe3.into_push_pull_output();
    //
    // reset.set_low();
    //
    // delay.delay_us(10);
    // reset.set_high();
    // delay.delay_ms(5);
    //reset end

    let mut led = gpioa.pa6.into_push_pull_output(); // just for notification, off when writing
                                                     // to LCD
    led.set_high();
    // delay.delay(5.secs());
    // delay.delay_ms(5);
    // controller.software_reset();
    // delay.delay_ms(120);

    // controller.sleep_out();
    // delay.delay_ms(100);
    // controller.display(false);
    // delay.delay_ms(5);

    // delay.delay(5.secs());
    // delay.delay_ms(5);

    // delay.delay(5.secs());

    let mut controller = lcd_ili9341::Controller::new(lcd);
    defmt::println!("reset");
    controller.software_reset();
    controller.memory_access_control(MemoryAccessControl::default());
    // controller.memory_access_control(MemoryAccessControl::to_check());
    controller.pixel_format_set(PixelFormat::bit16());
    controller.sleep_out();
    controller.display(true);
    controller.column_address_set(0, 320);
    controller.page_address_set(0, 240);
    let pixels = 240 * 320;

    delay.delay_ms(50);
    led.set_low();
    delay.delay_ms(50);
    led.set_high();
    delay.delay_ms(50);
    led.set_low();

    defmt::println!("done");

    defmt::println!("check {}", 0b1111100000000000 & (1));
    // rprintln!("done");
    loop {
        controller.memory_write_start();
        controller.write_memory(core::iter::repeat(0b0000100000000000).take(pixels));

        delay.delay_ms(4000);

        controller.memory_write_start();
        controller.write_memory(core::iter::repeat(0b0000000001100000).take(pixels));

        delay.delay_ms(4000);

        controller.memory_write_start();
        controller.write_memory(core::iter::repeat(0b0000000000011111).take(pixels));

        delay.delay_ms(4000);
    }
}
