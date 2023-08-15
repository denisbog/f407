#![no_std]

use core::convert::Infallible;

use embedded_hal::digital::v2::OutputPin;
use lcd_ili9341::Interface;
use rtt_target::rprintln;

pub trait Pins {
    type CSX: OutputPin<Error = Infallible>;
    type RESX: OutputPin<Error = Infallible>;
    type DCX: OutputPin<Error = Infallible>;
    type WRX: OutputPin<Error = Infallible>;
    type RDX: OutputPin<Error = Infallible>;
    type D0: OutputPin<Error = Infallible>;
    type D1: OutputPin<Error = Infallible>;
    type D2: OutputPin<Error = Infallible>;
    type D3: OutputPin<Error = Infallible>;
    type D4: OutputPin<Error = Infallible>;
    type D5: OutputPin<Error = Infallible>;
    type D6: OutputPin<Error = Infallible>;
    type D7: OutputPin<Error = Infallible>;
    type D8: OutputPin<Error = Infallible>;
    type D9: OutputPin<Error = Infallible>;
    type D10: OutputPin<Error = Infallible>;
    type D11: OutputPin<Error = Infallible>;
    type D12: OutputPin<Error = Infallible>;
    type D13: OutputPin<Error = Infallible>;
    type D14: OutputPin<Error = Infallible>;
    type D15: OutputPin<Error = Infallible>;
}

pub struct LCD<
    CSX,
    RESX,
    DCX,
    WRX,
    RDX,
    D0,
    D1,
    D2,
    D3,
    D4,
    D5,
    D6,
    D7,
    D8,
    D9,
    D10,
    D11,
    D12,
    D13,
    D14,
    D15,
> {
    pub csx: CSX,
    pub resx: RESX,
    pub dcx: DCX,
    pub wrx: WRX,
    pub rdx: RDX,
    pub d0: D0,
    pub d1: D1,
    pub d2: D2,
    pub d3: D3,
    pub d4: D4,
    pub d5: D5,
    pub d6: D6,
    pub d7: D7,
    pub d8: D8,
    pub d9: D9,
    pub d10: D10,
    pub d11: D11,
    pub d12: D12,
    pub d13: D13,
    pub d14: D14,
    pub d15: D15,
}

impl<
        CSX: OutputPin<Error = Infallible>,
        RESX: OutputPin<Error = Infallible>,
        DCX: OutputPin<Error = Infallible>,
        WRX: OutputPin<Error = Infallible>,
        RDX: OutputPin<Error = Infallible>,
        D0: OutputPin<Error = Infallible>,
        D1: OutputPin<Error = Infallible>,
        D2: OutputPin<Error = Infallible>,
        D3: OutputPin<Error = Infallible>,
        D4: OutputPin<Error = Infallible>,
        D5: OutputPin<Error = Infallible>,
        D6: OutputPin<Error = Infallible>,
        D7: OutputPin<Error = Infallible>,
        D8: OutputPin<Error = Infallible>,
        D9: OutputPin<Error = Infallible>,
        D10: OutputPin<Error = Infallible>,
        D11: OutputPin<Error = Infallible>,
        D12: OutputPin<Error = Infallible>,
        D13: OutputPin<Error = Infallible>,
        D14: OutputPin<Error = Infallible>,
        D15: OutputPin<Error = Infallible>,
    > Interface
    for LCD<
        CSX,
        RESX,
        DCX,
        WRX,
        RDX,
        D0,
        D1,
        D2,
        D3,
        D4,
        D5,
        D6,
        D7,
        D8,
        D9,
        D10,
        D11,
        D12,
        D13,
        D14,
        D15,
    >
{
    fn write_parameters(&mut self, command: u8, data: &[u8]) {
        self.csx.set_low().unwrap();
        self.rdx.set_high().unwrap();
        self.dcx.set_low().unwrap();
        rprintln!("writing {}", command);
        set_bit(command, 1, &mut self.d0);
        set_bit(command, 1 << 1, &mut self.d1);
        set_bit(command, 1 << 2, &mut self.d2);
        set_bit(command, 1 << 3, &mut self.d3);
        set_bit(command, 1 << 4, &mut self.d4);
        set_bit(command, 1 << 5, &mut self.d5);
        set_bit(command, 1 << 6, &mut self.d6);
        set_bit(command, 1 << 7, &mut self.d7);
        // self.wrx.set_low().unwrap();
        // self.wrx.set_high().unwrap();

        self.dcx.set_high().unwrap();

        if !data.is_empty() {
            data.iter().enumerate().for_each(|(index, &data)| {
                if index % 2 == 0 {
                    rprintln!("write first");
                    set_bit(data, 1, &mut self.d0);
                    set_bit(data, 1 << 1, &mut self.d1);
                    set_bit(data, 1 << 2, &mut self.d2);
                    set_bit(data, 1 << 3, &mut self.d3);
                    set_bit(data, 1 << 4, &mut self.d4);
                    set_bit(data, 1 << 5, &mut self.d5);
                    set_bit(data, 1 << 6, &mut self.d6);
                    set_bit(data, 1 << 7, &mut self.d7);
                } else {
                    rprintln!("write last");
                    set_bit(data, 1, &mut self.d8);
                    set_bit(data, 1 << 1, &mut self.d9);
                    set_bit(data, 1 << 2, &mut self.d10);
                    set_bit(data, 1 << 3, &mut self.d11);
                    set_bit(data, 1 << 4, &mut self.d12);
                    set_bit(data, 1 << 5, &mut self.d13);
                    set_bit(data, 1 << 6, &mut self.d14);
                    set_bit(data, 1 << 7, &mut self.d15);
                    // self.wrx.set_low().unwrap();
                    // self.wrx.set_high().unwrap();
                }
            });
        }
        if data.len() % 2 == 1 {
            // self.wrx.set_low().unwrap();
            // self.wrx.set_high().unwrap();
        }
        self.rdx.set_low().unwrap();
        self.csx.set_high().unwrap();
    }

    fn write_memory<I>(&mut self, iterable: I)
    where
        I: IntoIterator<Item = u32>,
    {
        self.csx.set_low().unwrap();
        self.rdx.set_high().unwrap();
        self.dcx.set_high().unwrap();
        iterable.into_iter().for_each(|data| {
            let temp = data as u16;
            set_16_bit(temp, 1, &mut self.d0);
            set_16_bit(temp, 1 << 1, &mut self.d1);
            set_16_bit(temp, 1 << 2, &mut self.d2);
            set_16_bit(temp, 1 << 3, &mut self.d3);
            set_16_bit(temp, 1 << 4, &mut self.d4);
            set_16_bit(temp, 1 << 5, &mut self.d5);
            set_16_bit(temp, 1 << 6, &mut self.d6);
            set_16_bit(temp, 1 << 7, &mut self.d7);
            set_16_bit(temp, 1 << 8, &mut self.d8);
            set_16_bit(temp, 1 << 9, &mut self.d9);
            set_16_bit(temp, 1 << 10, &mut self.d10);
            set_16_bit(temp, 1 << 11, &mut self.d11);
            set_16_bit(temp, 1 << 12, &mut self.d12);
            set_16_bit(temp, 1 << 13, &mut self.d13);
            set_16_bit(temp, 1 << 14, &mut self.d14);
            set_16_bit(temp, 1 << 15, &mut self.d15);
            rprintln!("write half");
            // self.wrx.set_low().unwrap();
            // self.wrx.set_high().unwrap();

            let temp = (data >> 16) as u16;
            set_16_bit(temp, 1, &mut self.d0);
            set_16_bit(temp, 1 << 1, &mut self.d1);
            set_16_bit(temp, 1 << 2, &mut self.d2);
            set_16_bit(temp, 1 << 3, &mut self.d3);
            set_16_bit(temp, 1 << 4, &mut self.d4);
            set_16_bit(temp, 1 << 5, &mut self.d5);
            set_16_bit(temp, 1 << 6, &mut self.d6);
            set_16_bit(temp, 1 << 7, &mut self.d7);
            set_16_bit(temp, 1 << 8, &mut self.d8);
            set_16_bit(temp, 1 << 9, &mut self.d9);
            set_16_bit(temp, 1 << 10, &mut self.d10);
            set_16_bit(temp, 1 << 11, &mut self.d11);
            set_16_bit(temp, 1 << 12, &mut self.d12);
            set_16_bit(temp, 1 << 13, &mut self.d13);
            set_16_bit(temp, 1 << 14, &mut self.d14);
            set_16_bit(temp, 1 << 15, &mut self.d15);
            rprintln!("write half");
            // self.wrx.set_low().unwrap();
            // self.wrx.set_high().unwrap();
        });
        self.dcx.set_low().unwrap();
        self.csx.set_high().unwrap();
    }

    fn read_parameters(&mut self, _command: u8, _data: &mut [u8]) {}

    fn read_memory(&mut self, _data: &mut [u32]) {}
}

fn set_bit<P: OutputPin<Error = Infallible>>(command: u8, mask: u8, pin: &mut P) {
    rprintln!("mask {}", command & mask);
    if command & mask > 0 {
        rprintln!("set high");
        pin.set_high().unwrap();
    } else {
        rprintln!("set low");
        pin.set_low().unwrap();
    }
}

fn set_16_bit<P: OutputPin<Error = Infallible>>(data: u16, mask: u16, pin: &mut P) {
    if data & mask > 0 {
        pin.set_high().unwrap();
    } else {
        pin.set_low().unwrap();
    }
}
