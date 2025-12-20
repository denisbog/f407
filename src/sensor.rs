use stm32f4xx_hal::{
    gpio::{OpenDrain, Output, Pin},
    hal::digital::InputPin,
    pac::DWT,
};

#[inline(always)]
fn delay_us(us: u32, sysclk_hz: u32) {
    let start = DWT::cycle_count();
    let ticks = us * (sysclk_hz / 1_000_000);
    // defmt::println!("start wait");
    while DWT::cycle_count().wrapping_sub(start) < ticks {}
    // defmt::println!("end wait");
}

pub(crate) fn read_bit<P: InputPin>(pin: &mut P, sysclk: u32) -> u8 {
    // wait for LOW (50µs)
    while pin.is_high().unwrap() {}
    // wait for HIGH start
    while pin.is_low().unwrap() {}
    let start = DWT::cycle_count();
    // wait for HIGH end
    while pin.is_high().unwrap() {}
    let dur = DWT::cycle_count().wrapping_sub(start);
    // threshold ~50µs
    if dur > (sysclk / 1_000_000 * 50) {
        1
    } else {
        0
    }
}

pub fn read_dht21<const P: char, const N: u8>(
    pin: &mut Pin<P, N, Output<OpenDrain>>,
    sysclk: u32,
) -> Result<(f32, f32), &'static str> {
    // Start signal
    pin.set_low();
    delay_us(1200, sysclk);
    pin.set_high();

    let mut data = [0u8; 5];
    // Switch to input
    // let mut pin = pin.into_pull_up_input();
    pin.with_input(|pin| {
        // Sensor response
        delay_us(40, sysclk);
        if pin.is_high().unwrap() {
            return Err("No response");
        }

        // skip 80µs low + 80µs high
        while pin.is_low().unwrap() {}
        while pin.is_high().unwrap() {}

        for byte in 0..5 {
            for _bit in 0..8 {
                data[byte] <<= 1;
                data[byte] |= read_bit(pin, sysclk);
            }
        }
        // checksum
        if data[4]
            != data[0]
                .wrapping_add(data[1])
                .wrapping_add(data[2])
                .wrapping_add(data[3])
        {
            return Err("Checksum error");
        }
        let humidity = ((data[0] as u16) << 8 | data[1] as u16) as f32 / 10.0;
        let mut temperature = ((data[2] as u16 & 0x7F) << 8 | data[3] as u16) as f32 / 10.0;
        if data[2] & 0x80 != 0 {
            temperature = -temperature;
        }
        Ok((temperature, humidity))
    })
}
