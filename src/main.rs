#![no_main]
#![no_std]

use cortex_m::interrupt::Mutex;
use defmt_rtt as _;
use embedded_graphics::primitives::Rectangle;
use f407::sensor::read_dht21;
use heapless::String;
use ili9341::Orientation;
use panic_halt as _;
use stm32f4xx_hal::gpio::alt::fsmc;
use stm32f4xx_hal::{
    dwt::DwtExt,
    fsmc_lcd::{DataPins16, FsmcLcd, LcdPins, Timing},
    gpio::{Edge, ExtiPin, GpioExt, Input, PE3},
    interrupt,
    pac::{Peripherals, TIM3},
    prelude::*,
    rcc::{Config, RccExt},
    serial::SerialExt,
    timer::{PwmChannel, TimerExt, C4},
};

use core::{
    cell::{Cell, RefCell},
    fmt::Write,
};
use cortex_m_rt::entry;

type ButtonPin = PE3<Input>;

static BACKLIT_BUTTON: Mutex<RefCell<Option<ButtonPin>>> = Mutex::new(RefCell::new(None));
static BACKLIT_CHANNEL: Mutex<RefCell<Option<PwmChannel<TIM3, C4>>>> =
    Mutex::new(RefCell::new(None));
static BACKLIT_CURRENT_LEVEL: Mutex<Cell<u16>> = Mutex::new(Cell::new(8u16));

// Blink state management
static BLINK_STATE: Mutex<Cell<bool>> = Mutex::new(Cell::new(false));
static BLINK_ENABLED: Mutex<Cell<bool>> = Mutex::new(Cell::new(false));
static SENSOR_ERROR: Mutex<Cell<bool>> = Mutex::new(Cell::new(false));
static MAIN_COUNTER: Mutex<Cell<u32>> = Mutex::new(Cell::new(0));

#[interrupt]
fn EXTI3() {
    defmt::info!("change backlit intensity");
    cortex_m::interrupt::free(|cs| {
        BACKLIT_BUTTON
            .borrow(cs)
            .borrow_mut()
            .as_mut()
            .unwrap()
            .clear_interrupt_pending_bit();
        if let Some(backlit) = BACKLIT_CHANNEL.borrow(cs).borrow_mut().as_mut() {
            let mut current_level = BACKLIT_CURRENT_LEVEL.borrow(cs).get();
            if current_level < backlit.get_max_duty() {
                current_level *= 2;
            } else {
                current_level = 1
            };
            backlit.set_duty(backlit.get_max_duty() / current_level);
            BACKLIT_CURRENT_LEVEL.borrow(cs).replace(current_level);
            defmt::info!("current level {} ", BACKLIT_CURRENT_LEVEL.borrow(cs).get());
        }
    });
}

#[interrupt]
fn TIM4() {
    cortex_m::interrupt::free(|cs| {
        // Clear interrupt flag
        let timer = unsafe { &*stm32f4xx_hal::pac::TIM4::ptr() };
        timer.sr().write(|w| w.uif().clear_bit());

        // Toggle blink state every second
        let blink_state = BLINK_STATE.borrow(cs);
        let current = blink_state.get();
        blink_state.set(!current);

        // Increment main counter for sensor reading timing
        let counter = MAIN_COUNTER.borrow(cs);
        counter.set(counter.get() + 1);
    });
}
#[entry]
fn main() -> ! {
    let mut dp = Peripherals::take().unwrap();
    let mut rcc = dp.RCC.freeze(Config::hsi().sysclk(48.MHz()).pclk1(8.MHz()));
    // let mut rcc = dp
    //     .RCC
    //     .freeze(Config::hse(8.MHz()).sysclk(48.MHz()).pclk1(8.MHz()));
    // let mut rcc = dp.RCC.freeze(
    //     Config::hse(8.MHz())
    //         .sysclk(168.MHz())
    //         .pclk1(8.MHz())
    //         .pclk2(8.MHz()),
    // );
    let cp = cortex_m::peripheral::Peripherals::take().unwrap();
    let dwt = cp.DWT.constrain(cp.DCB, &rcc.clocks);
    let mut local_timer = dwt.delay();

    defmt::println!("led display");
    let gpiod = dp.GPIOD.split(&mut rcc);
    let gpioe = dp.GPIOE.split(&mut rcc);
    let gpiob = dp.GPIOB.split(&mut rcc);

    // Setup TIM3 for backlight PWM (existing)
    let (_, (_, _, _, ch4, ..)) = dp.TIM3.pwm_us(100.micros(), &mut rcc);
    let mut ch4: PwmChannel<_, _> = ch4.with(gpiob.pb1);
    cortex_m::interrupt::free(|cs| {
        ch4.set_duty(ch4.get_max_duty() / BACKLIT_CURRENT_LEVEL.borrow(cs).get());
    });
    ch4.enable();

    // Setup TIM4 for 1Hz interrupt for blinking
    let mut timer4 = dp.TIM4.counter_hz(&mut rcc);
    timer4.start(1.Hz()).unwrap();
    timer4.listen(stm32f4xx_hal::timer::Event::Update);
    unsafe {
        cortex_m::peripheral::NVIC::unmask(stm32f4xx_hal::pac::Interrupt::TIM4);
    }
    let mut button = gpioe.pe3.internal_pull_up(true);
    let mut syscfg = dp.SYSCFG.constrain(&mut rcc);
    button.make_interrupt_source(&mut syscfg);
    button.trigger_on_edge(&mut dp.EXTI, Edge::Rising);
    button.enable_interrupt(&mut dp.EXTI);

    unsafe {
        cortex_m::peripheral::NVIC::unmask(button.interrupt());
    }

    cortex_m::interrupt::free(|cs| {
        BACKLIT_BUTTON.borrow(cs).replace(Some(button));
        BACKLIT_CHANNEL.borrow(cs).replace(Some(ch4));
    });

    // Set up timing
    let write_timing = Timing::default().data(3).address_setup(3).bus_turnaround(0);
    let read_timing = Timing::default().data(8).address_setup(8).bus_turnaround(0);

    let lcd_pins = LcdPins::new(
        DataPins16::new(
            gpiod.pd14, gpiod.pd15, gpiod.pd0, gpiod.pd1, gpioe.pe7, gpioe.pe8, gpioe.pe9,
            gpioe.pe10, gpioe.pe11, gpioe.pe12, gpioe.pe13, gpioe.pe14, gpioe.pe15, gpiod.pd8,
            gpiod.pd9, gpiod.pd10,
        ),
        fsmc::Address::from(gpiod.pd13),
        gpiod.pd4,
        gpiod.pd5,
        fsmc::ChipSelect1::from(gpiod.pd7),
    );

    // Initialise FSMC memory provider
    let (_fsmc, interface) = FsmcLcd::new(dp.FSMC, lcd_pins, &read_timing, &write_timing, &mut rcc);
    defmt::println!("lcd");
    let reset = gpioe.pe5.into_push_pull_output();
    let mut delay = dp.TIM2.delay_ms(&mut rcc);
    defmt::println!("controller");
    let mut controller = ili9341::Ili9341::new(
        interface,
        reset,
        &mut delay,
        Orientation::Landscape,
        ili9341::DisplaySize240x320,
    )
    .unwrap();
    defmt::println!("loop");

    use embedded_graphics::{
        mono_font::{ascii::FONT_7X14, MonoTextStyle},
        pixelcolor::Rgb565,
        prelude::*,
        text::Text,
    };

    let gpioa = dp.GPIOA.split(&mut rcc);
    let mut sensor = gpioa.pa8.into_open_drain_output().internal_pull_up(true);
    sensor.set_high();
    let tx_pin = gpioa.pa9;

    let mut tx = dp.USART1.tx(tx_pin, 9600.bps(), &mut rcc).unwrap();
    writeln!(tx, "waiting data.").unwrap();

    // Create a new character style
    let style = MonoTextStyle::new(&FONT_7X14, Rgb565::WHITE);
    controller.clear(Rgb565::RED).unwrap();
    Text::new("Hello Rust! Wait a second..", Point::new(20, 30), style)
        .draw(&mut controller)
        .unwrap();
    local_timer.delay_ms(1000);
    controller.clear(Rgb565::WHITE).unwrap();
    let overwrite = &Rectangle::new(Point::new(18, 15), Size::new(150, 20));

    use embedded_graphics_framebuf::FrameBuf;

    let mut last_blink_state = false;
    let mut last_sensor_read_count = 0u32;

    loop {
        // Check if we need to read sensor (every 2 seconds based on main counter)
        let (should_read_sensor, counter_value) = cortex_m::interrupt::free(|cs| {
            let counter = MAIN_COUNTER.borrow(cs).get();
            let should_read = counter - last_sensor_read_count >= 2;
            (should_read, counter)
        });

        if should_read_sensor {
            last_sensor_read_count = counter_value;

            cortex_m::interrupt::free(|_| {
                let data = read_dht21(&mut sensor, rcc.clocks.sysclk().raw());
                if let Ok((temp, humidity)) = data {
                    // Sensor reading successful - disable blinking
                    cortex_m::interrupt::free(|cs| {
                        SENSOR_ERROR.borrow(cs).set(false);
                        BLINK_ENABLED.borrow(cs).set(false);
                    });

                    let mut buf_data = [<Rgb565 as RgbColor>::WHITE; 150 * 20];
                    let mut fbuf = FrameBuf::new(&mut buf_data, 150, 20);
                    fbuf.fill_solid(
                        &Rectangle::new(Point::new(0, 0), Size::new(150, 20)),
                        Rgb565::BLUE,
                    )
                    .unwrap();
                    defmt::info!("data {} {}", temp, humidity);
                    let mut s: String<64> = String::new();
                    write!(s, "Tem {} Hum {} !!", temp, humidity).unwrap();
                    Text::new(&s, Point::new(4, 14), style)
                        .draw(&mut fbuf)
                        .unwrap();
                    writeln!(tx, "{} {}", temp, humidity).unwrap();
                    controller.fill_contiguous(overwrite, buf_data).unwrap();
                } else {
                    // Sensor error - enable blinking
                    defmt::error!("failure to read data");
                    writeln!(tx, "no data").unwrap();
                    cortex_m::interrupt::free(|cs| {
                        SENSOR_ERROR.borrow(cs).set(true);
                        BLINK_ENABLED.borrow(cs).set(true);
                    });
                }
            });
        }

        // Check if display needs update due to blinking
        let (blink_enabled, blink_state, sensor_error) = cortex_m::interrupt::free(|cs| {
            (
                BLINK_ENABLED.borrow(cs).get(),
                BLINK_STATE.borrow(cs).get(),
                SENSOR_ERROR.borrow(cs).get(),
            )
        });

        if blink_enabled && sensor_error && blink_state != last_blink_state {
            last_blink_state = blink_state;

            // Update display with blink effect
            let mut buf_data = [<Rgb565 as RgbColor>::WHITE; 150 * 20];
            let mut s: String<64> = String::new();
            write!(s, "<no sensor data>").unwrap();

            let background_color = if blink_state {
                Rgb565::YELLOW
            } else {
                Rgb565::WHITE
            };

            let text_color = if blink_state {
                Rgb565::WHITE
            } else {
                Rgb565::BLACK
            };

            let blink_style = MonoTextStyle::new(&FONT_7X14, text_color);

            let mut fbuf = FrameBuf::new(&mut buf_data, 150, 20);
            fbuf.fill_solid(
                &Rectangle::new(Point::new(0, 0), Size::new(150, 20)),
                background_color,
            )
            .unwrap();

            Text::new(&s, Point::new(4, 14), blink_style)
                .draw(&mut fbuf)
                .unwrap();
            controller.fill_contiguous(overwrite, buf_data).unwrap();
        }

        // Small delay to prevent busy-waiting
        local_timer.delay_ms(10);
    }
}
