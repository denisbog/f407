#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_halt as _;

#[rtic::app(device = stm32f4xx_hal::pac, peripherals = true)]
mod app {
    use stm32f4xx_hal::{
        gpio::{gpioa::PA6, gpioe::PE4, Edge, Input, Output, PushPull},
        prelude::*,
    };
    const SYSFREQ: u32 = 100_000_000;
    // Shared resources go here
    #[shared]
    struct Shared {}

    // Local resources go here
    #[local]
    struct Local {
        button: PE4<Input>,
        led: PA6<Output<PushPull>>,
    }

    #[init]
    fn init(mut ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        // syscfg
        let mut syscfg = ctx.device.SYSCFG.constrain();
        // clocks
        let rcc = ctx.device.RCC.constrain();
        let _clocks = rcc.cfgr.sysclk(SYSFREQ.Hz()).use_hse(25.MHz()).freeze();
        // gpio ports A and C
        let gpioe = ctx.device.GPIOE.split();
        let gpioa = ctx.device.GPIOA.split();
        // button
        let mut button = gpioe.pe4.into_pull_up_input();
        button.make_interrupt_source(&mut syscfg);
        button.enable_interrupt(&mut ctx.device.EXTI);
        button.trigger_on_edge(&mut ctx.device.EXTI, Edge::Rising);
        // led
        let led = gpioa.pa6.into_push_pull_output();

        (
            Shared {
               // Initialization of shared resources go here
            },
            Local {
                // Initialization of local resources go here
                button,
                led,
            },
            init::Monotonics(),
        )
    }

    // Optional idle, can be removed if not needed.
    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            continue;
        }
    }

    #[task(binds = EXTI4, local = [button, led])]
    fn button_click(ctx: button_click::Context) {
        ctx.local.button.clear_interrupt_pending_bit();
        ctx.local.led.toggle();
    }
}
