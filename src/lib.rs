#![no_std]

use core::convert::Infallible;

use display_interface::WriteOnlyDataCommand;
use stm32f4xx_hal::{
    hal::digital::OutputPin,
    prelude::*,
    timer::{Delay, Instance},
};

pub struct LCD<
    'a,
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
    T,
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
    pub delay: &'a mut Delay<T, 1000000>,
}

impl<
        'a,
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
        T: Instance,
    >
    LCD<
        'a,
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
        T,
    >
{
    fn write(&mut self) {
        self.wrx.set_low().unwrap();
        self.delay.delay(15.nanos());
        self.wrx.set_high().unwrap();
        self.delay.delay(10.nanos());
    }
    pub fn reset(&mut self) {
        self.delay.delay(300.nanos());
        self.resx.set_low().unwrap();
        self.delay.delay(300.nanos());
        self.resx.set_high().unwrap();
        self.delay.delay(300.nanos());
    }
}
impl<
        'a,
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
        T: Instance,
    > WriteOnlyDataCommand
    for LCD<
        'a,
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
        T,
    >
{
    fn send_commands(
        &mut self,
        cmd: display_interface::DataFormat<'_>,
    ) -> Result<(), ili9341::DisplayError> {
        self.csx.set_low().unwrap();
        self.rdx.set_high().unwrap();
        self.dcx.set_low().unwrap();
        let out = match cmd {
            display_interface::DataFormat::U8Iter(commands) => {
                for (index, command) in commands.enumerate() {
                    defmt::println!("write command {} {:#02x}", index, command);
                    set_bit(command, 1, &mut self.d0);
                    set_bit(command, 1 << 1, &mut self.d1);
                    set_bit(command, 1 << 2, &mut self.d2);
                    set_bit(command, 1 << 3, &mut self.d3);
                    set_bit(command, 1 << 4, &mut self.d4);
                    set_bit(command, 1 << 5, &mut self.d5);
                    set_bit(command, 1 << 6, &mut self.d6);
                    set_bit(command, 1 << 7, &mut self.d7);
                    self.write();
                }
                Ok(())
            }
            display_interface::DataFormat::U8(commands) => {
                for (index, &command) in commands.iter().enumerate() {
                    defmt::println!("write commands {} {:#02x}", index, command);
                    set_bit(command, 1, &mut self.d0);
                    set_bit(command, 1 << 1, &mut self.d1);
                    set_bit(command, 1 << 2, &mut self.d2);
                    set_bit(command, 1 << 3, &mut self.d3);
                    set_bit(command, 1 << 4, &mut self.d4);
                    set_bit(command, 1 << 5, &mut self.d5);
                    set_bit(command, 1 << 6, &mut self.d6);
                    set_bit(command, 1 << 7, &mut self.d7);
                    self.write();
                }
                Ok(())
            }
            _ => {
                defmt::println!("error sending command");
                Err(ili9341::DisplayError::BusWriteError)
            }
        };
        self.csx.set_high().unwrap();
        out
    }

    fn send_data(
        &mut self,
        buf: display_interface::DataFormat<'_>,
    ) -> Result<(), ili9341::DisplayError> {
        self.csx.set_low().unwrap();
        self.rdx.set_high().unwrap();
        self.dcx.set_high().unwrap();
        let out = match buf {
            display_interface::DataFormat::U8Iter(iterable) => {
                iterable.into_iter().enumerate().for_each(|(index, data)| {
                    defmt::println!(
                        "write data {} {:#04x} ({}) {:#010b}",
                        index + 1,
                        data,
                        data,
                        data
                    );
                    set_bit(data, 1, &mut self.d0);
                    set_bit(data, 1 << 1, &mut self.d1);
                    set_bit(data, 1 << 2, &mut self.d2);
                    set_bit(data, 1 << 3, &mut self.d3);
                    set_bit(data, 1 << 4, &mut self.d4);
                    set_bit(data, 1 << 5, &mut self.d5);
                    set_bit(data, 1 << 6, &mut self.d6);
                    set_bit(data, 1 << 7, &mut self.d7);
                    self.write();
                });
                Ok(())
            }
            display_interface::DataFormat::U16BEIter(iterable) => {
                iterable.into_iter().for_each(|data| {
                    set_16_bit(data, 1, &mut self.d0);
                    set_16_bit(data, 1 << 1, &mut self.d1);
                    set_16_bit(data, 1 << 2, &mut self.d2);
                    set_16_bit(data, 1 << 3, &mut self.d3);
                    set_16_bit(data, 1 << 4, &mut self.d4);
                    set_16_bit(data, 1 << 5, &mut self.d5);
                    set_16_bit(data, 1 << 6, &mut self.d6);
                    set_16_bit(data, 1 << 7, &mut self.d7);
                    set_16_bit(data, 1 << 8, &mut self.d8);
                    set_16_bit(data, 1 << 9, &mut self.d9);
                    set_16_bit(data, 1 << 10, &mut self.d10);
                    set_16_bit(data, 1 << 11, &mut self.d11);
                    set_16_bit(data, 1 << 12, &mut self.d12);
                    set_16_bit(data, 1 << 13, &mut self.d13);
                    set_16_bit(data, 1 << 14, &mut self.d14);
                    set_16_bit(data, 1 << 15, &mut self.d15);
                    self.write();
                });
                Ok(())
            }
            _ => {
                defmt::println!("error sending data");
                Err(ili9341::DisplayError::BusWriteError)
            }
        };
        self.csx.set_high().unwrap();
        out
    }
}

fn set_bit<P: OutputPin<Error = Infallible>>(command: u8, mask: u8, pin: &mut P) {
    if command & mask > 0 {
        pin.set_high().unwrap();
    } else {
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
