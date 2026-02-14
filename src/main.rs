#![no_std]
#![no_main]

use cortex_m::prelude::_embedded_hal_Pwm;
use crc::{Crc, CRC_8_NRSC_5};
use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::select::{select, Either};
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Level, Output, Pull, Speed};
use embassy_stm32::rcc::{self, Hse, HseMode, Pll, PllMul, PllPreDiv, Sysclk};
use embassy_stm32::time::{khz, Hertz};
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::timer::Channel;
use embassy_stm32::{bind_interrupts, i2c, peripherals};
use embassy_stm32_fsmc_display_interface::{FsmcLcd, Timing};
use embassy_time::Instant;
use embassy_time::{Delay, Duration, Timer};
use embedded_aht20::{Aht20, DEFAULT_I2C_ADDRESS};

use embassy_stm32::i2c::{Config as I2cConfig, I2c};
use ili9341::{Ili9341, Orientation};
use panic_probe as _;

use {defmt_rtt as _, panic_probe as _};

// Display dimensions
const DISPLAY_WIDTH: u16 = 320;
const DISPLAY_HEIGHT: u16 = 240;

use embedded_graphics::mono_font::{ascii::FONT_10X20, MonoTextStyle};
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::text::Text;
use embedded_graphics::{pixelcolor::Rgb565, prelude::*};
use heapless::String;

/// Text region that can be drawn with background clearing
struct TextRegion {
    text: String<32>,
    position: Point,
    clear_box: Rectangle, // Fixed size for clearing (max possible text)
    max_chars: usize,
}

impl TextRegion {
    fn new(position: Point, max_chars: usize) -> Self {
        // Calculate fixed bounding box for max text - always clear this area
        let char_width = 10; // FONT_10X20 width
        let char_height = 20; // FONT_10X20 height
        let width = (max_chars * char_width) as u32;
        let height = char_height as u32;
        let box_position = Point::new(position.x, position.y - height as i32);
        Self {
            text: String::new(),
            position,
            clear_box: Rectangle::new(box_position, Size::new(width + 1, height + 1)),
            max_chars,
        }
    }

    fn set_text(&mut self, text: &str) {
        self.text.clear();
        // Truncate if too long
        let to_copy = text.len().min(self.max_chars);
        self.text.push_str(&text[..to_copy]).ok();
    }

    fn draw<D: DrawTarget<Color = Rgb565>>(
        &self,
        target: &mut D,
        style: MonoTextStyle<Rgb565>,
        bg_color: Rgb565,
    ) -> Result<(), D::Error> {
        // Clear the entire fixed region first (to erase any previous longer text)
        self.clear_box
            .into_styled(embedded_graphics::primitives::PrimitiveStyle::with_fill(
                bg_color,
            ))
            .draw(target)?;
        // Draw new text
        Text::new(&self.text, self.position, style).draw(target)?;
        Ok(())
    }
}
bind_interrupts!(struct Irqs {
    I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
    I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
});
/// Hardware configuration for STM32F407 + ILI9341 using FSMC 8080 interface
///
/// Pin mapping for 16-bit parallel 8080 interface:
/// - PD7  -> FSMC_NE1   (Chip Select, CS)
/// - PD4  -> FSMC_NOE   (Read Enable, RD)
/// - PD5  -> FSMC_NWE   (Write Enable, WR)
/// - PD13 -> FSMC_A18   (Register Select/Data-Command, RS/DC)
/// - PD14, PD15, PD0, PD1 -> FSMC_D0-D3  (Data lines)
/// - PE7-PE15 -> FSMC_D4-D12             (Data lines)
/// - PD8-PD10 -> FSMC_D13-D15            (Data lines)
/// - PB0 -> RESET (GPIO output, consumed by Ili9341 driver)

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("=== STM32F407 ILI9341 LCD Display ===");
    info!("Starting application...");

    // Configure clocks for maximum performance
    // STM32F407 HSE = 8MHz, target SYSCLK = 168MHz
    let mut config = embassy_stm32::Config::default();
    config.rcc.hse = Some(Hse {
        freq: embassy_stm32::time::Hertz(8_000_000),
        mode: HseMode::Oscillator,
    });
    config.rcc.pll_src = rcc::PllSource::HSE;
    config.rcc.pll = Some(Pll {
        prediv: PllPreDiv::DIV4,        // 8MHz / 4 = 2MHz
        mul: PllMul::MUL168,            // 2MHz * 168 = 336MHz
        divp: Some(rcc::PllPDiv::DIV2), // 336MHz / 2 = 168MHz (SYSCLK)
        divq: Some(rcc::PllQDiv::DIV7), // 336MHz / 7 = 48MHz (USB)
        divr: None,
    });
    config.rcc.ahb_pre = rcc::AHBPrescaler::DIV1; // 168MHz
    config.rcc.apb1_pre = rcc::APBPrescaler::DIV4; // 42MHz
    config.rcc.apb2_pre = rcc::APBPrescaler::DIV2; // 84MHz
    config.rcc.sys = Sysclk::PLL1_P;

    // Initialize STM32 peripherals
    let p = embassy_stm32::init(config);

    info!("System initialized at 168MHz");

    // Configure reset pin as GPIO output
    let rst = Output::new(p.PB0, Level::High, Speed::High);

    // Configure PWM for backlight control on PB1 (TIM3_CH4)
    let pwm_pin = PwmPin::new(p.PB1, embassy_stm32::gpio::OutputType::PushPull);
    let mut pwm = SimplePwm::new(
        p.TIM3,
        None,          // CH1 not used
        None,          // CH2 not used
        None,          // CH3 not used
        Some(pwm_pin), // CH4 on PB1
        khz(20),       // 20kHz PWM frequency (good for LED, no flicker)
        Default::default(),
    );
    let backlight_channel = Channel::Ch4;

    // 5 intensity levels: 0%, 25%, 50%, 75%, 100%
    const INTENSITY_LEVELS: [u16; 5] = [0, 25, 50, 75, 100];
    let mut current_level_idx: usize = 1; // Start at 25%
    pwm.set_duty(
        backlight_channel,
        pwm.get_max_duty() * INTENSITY_LEVELS[current_level_idx] as u32 / 100,
    );
    pwm.enable(backlight_channel);
    info!(
        "Backlight PWM on PB1 (TIM3_CH4) configured at 20kHz, starting at {}%",
        INTENSITY_LEVELS[current_level_idx]
    );

    // Configure button input on PE3 with pull-up and external interrupt
    let mut button = ExtiInput::new(p.PE3, p.EXTI3, Pull::Up);
    info!("Button configured on PE3 with EXTI interrupt");

    // Create delay provider
    let mut delay = Delay;

    info!("Reset pin configured");

    // Initialize FMC peripheral for LCD
    info!("Initializing FSMC for 8080 interface...");

    let mut timing = Timing::default();
    timing.bus_turnaround = 1;
    timing.data = 4;
    timing.address_hold = 0;
    timing.address_setup = 0;

    // Create FSMC LCD interface with proper pin configuration
    // Uses Intel 8080 protocol via FSMC NOR/PSRAM Bank 1
    let lcd_interface = FsmcLcd::new(
        p.PD7,  // CS  (Chip Select / FSMC_NE1)
        p.PD4,  // RD  (Read Enable / FSMC_NOE)
        p.PD5,  // WR  (Write Enable / FSMC_NWE)
        p.PD13, // RS  (Register Select / FSMC_A18 - controls command/data)
        (
            p.PD14, p.PD15, p.PD0, p.PD1, // D0-D3
            p.PE7, p.PE8, p.PE9, p.PE10, // D4-D7
            p.PE11, p.PE12, p.PE13, p.PE14, // D8-D11
            p.PE15, p.PD8, p.PD9, p.PD10, // D12-D15
        ),
        &timing, // Read timing
        &timing, // Write timing
    );

    info!("FSMC interface created");

    // Initialize ILI9341 display driver
    info!("Initializing ILI9341 display driver...");

    let mut display = Ili9341::new(
        lcd_interface,
        rst,
        &mut delay,
        Orientation::Landscape,
        ili9341::DisplaySize240x320,
    )
    .expect("Failed to initialize display");

    info!("Display initialized successfully!");
    info!("Resolution: {}x{}", DISPLAY_WIDTH, DISPLAY_HEIGHT);

    // Initialize I2C1 for AHT20 sensor
    info!("Initializing I2C1 on PB6/PB7 for AHT20...");
    let mut i2c_config = I2cConfig::default();
    i2c_config.frequency = Hertz(100_000); // 100kHz standard mode
                                           // i2c_config.sda_pullup = true; // Enable internal pull-up on SDA
                                           // i2c_config.scl_pullup = true; // Enable internal pull-up on SCL
    let i2c = I2c::new(
        p.I2C1, p.PB6, p.PB7, Irqs, p.DMA1_CH6, p.DMA1_CH0, i2c_config,
    );
    info!("I2C1 initialized at 100kHz with internal pull-ups");

    // Initialize AHT20 sensor
    info!("Initializing AHT20 sensor...");
    let mut sensor = Aht20::new(i2c, DEFAULT_I2C_ADDRESS, Delay)
        .await
        .expect("Failed to initialize AHT20 sensor");
    info!("AHT20 sensor initialized!");

    // Create text style
    let text_style = MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE);
    let bg_color = Rgb565::BLACK;

    // Clear screen once at startup
    display.clear(bg_color).unwrap();

    info!("Starting display loop...");

    // Main loop - read sensor and display on screen
    let mut last_sensor_read = Instant::now();
    let sensor_interval = Duration::from_millis(200);
    let mut temp = 0.0f32;
    let mut humidity = 0.0f32;
    let mut sensor_ok = false;

    // Create text regions for partial updates
    let mut temp_region = TextRegion::new(Point::new(20, 100), 20);
    let mut hum_region = TextRegion::new(Point::new(20, 140), 20);
    let mut bl_region = TextRegion::new(Point::new(20, 180), 20);

    // Previous values to detect changes
    let mut prev_temp_str: String<32> = String::new();
    let mut prev_hum_str: String<32> = String::new();
    let mut prev_bl_str: String<32> = String::new();

    // Initial draw
    let mut need_full_redraw = true;

    loop {
        // Build current text strings
        let mut temp_str: String<32> = String::new();
        if sensor_ok {
            core::fmt::Write::write_fmt(&mut temp_str, format_args!("Temp: {:.1} C", temp))
                .unwrap();
        } else {
            temp_str.push_str("Temp: --").unwrap();
        }

        let mut hum_str: String<32> = String::new();
        if sensor_ok {
            core::fmt::Write::write_fmt(&mut hum_str, format_args!("Humidity: {:.1} %", humidity))
                .unwrap();
        } else {
            hum_str.push_str("Humidity: --").unwrap();
        }

        let mut bl_str: String<32> = String::new();
        core::fmt::Write::write_fmt(
            &mut bl_str,
            format_args!("Backlight: {}%", INTENSITY_LEVELS[current_level_idx]),
        )
        .unwrap();

        // Check what changed and redraw only changed regions
        if need_full_redraw {
            // First draw or sensor state changed - clear screen and draw all
            display.clear(bg_color).unwrap();

            temp_region.set_text(&temp_str);
            temp_region
                .draw(&mut display, text_style, bg_color)
                .unwrap();

            hum_region.set_text(&hum_str);
            hum_region.draw(&mut display, text_style, bg_color).unwrap();

            bl_region.set_text(&bl_str);
            bl_region.draw(&mut display, text_style, bg_color).unwrap();

            need_full_redraw = false;
        } else {
            // Only redraw changed regions
            if temp_str != prev_temp_str {
                temp_region.set_text(&temp_str);
                temp_region
                    .draw(&mut display, text_style, bg_color)
                    .unwrap();
            }

            if hum_str != prev_hum_str {
                hum_region.set_text(&hum_str);
                hum_region.draw(&mut display, text_style, bg_color).unwrap();
            }

            if bl_str != prev_bl_str {
                bl_region.set_text(&bl_str);
                bl_region.draw(&mut display, text_style, bg_color).unwrap();
            }
        }

        // Save current values
        prev_temp_str = temp_str;
        prev_hum_str = hum_str;
        prev_bl_str = bl_str;

        // Wait for either button press (interrupt) or timer tick
        let button_fut = button.wait_for_falling_edge();
        let timer_fut = Timer::after_millis(100);

        match select(button_fut, timer_fut).await {
            Either::First(_) => {
                // Button pressed - interrupt triggered on falling edge
                info!("Button interrupt triggered!");

                // Cycle to next intensity level
                current_level_idx = (current_level_idx + 1) % INTENSITY_LEVELS.len();
                let duty = pwm.get_max_duty() * INTENSITY_LEVELS[current_level_idx] as u32 / 100;
                pwm.set_duty(backlight_channel, duty);
                info!(
                    "Button pressed - backlight intensity: {}%",
                    INTENSITY_LEVELS[current_level_idx]
                );

                // Debounce delay
                Timer::after_millis(200).await;
            }
            Either::Second(_) => {
                // Timer tick - check if we need to read sensor
                if last_sensor_read.elapsed() >= sensor_interval {
                    last_sensor_read = Instant::now();

                    match sensor
                        .measure_crc(|data: &[u8], crc: u8| {
                            debug!("data: {}", data);
                            debug!("crc: {}", crc);
                            let crc_d = Crc::<u8>::new(&CRC_8_NRSC_5);
                            let mut digest = crc_d.digest();
                            digest.update(data);
                            if digest.finalize() != crc {
                                warn!("crc failed");
                            }
                            Ok(())
                        })
                        .await
                    {
                        Ok(measurement) => {
                            let new_temp = measurement.temperature.celsius();
                            let new_humidity = measurement.relative_humidity;

                            // Check if values actually changed
                            if (new_temp - temp).abs() > 0.05
                                || (new_humidity - humidity).abs() > 0.5
                                || !sensor_ok
                            {
                                temp = new_temp;
                                humidity = new_humidity;
                                sensor_ok = true;
                                info!("AHT20: Temperature = {}°C, Humidity = {}%", temp, humidity);
                            }
                        }
                        Err(_e) => {
                            if sensor_ok {
                                error!("AHT20 read error");
                                sensor_ok = false;
                                need_full_redraw = true;
                            }
                        }
                    }
                }
            }
        }
    }
}
