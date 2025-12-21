# doc

## flash on chip

```sh
cargo flash --release --chip STM32F407VGTx
```

update following lines in `.cargo/config.toml` to be able use cargo run command:

```sh
[target.'cfg(all(target_arch = "arm", target_os = "none"))']
runner = "probe-run --chip STM32F407VGTx"
```

## running the commad with the library

### log

```sh
DEFMT_LOG=warn cargo r --release

defmt::println!("write command {} {:#02x} ({})", index + 1, data, data);
```

```rust

defmt::println!(
    "write command {} {:#04x} ({}) {:#010b}",
    index + 1,
    data,
    data,
    data
);

```

### display interface with

```rust
//required dependency Cargo.toml # display-interface-parallel-gpio = "0.7.0"

    use display_interface_parallel_gpio::Generic16BitBus;
    use display_interface_parallel_gpio::PGPIO16BitInterface;

    let lcd = PGPIO16BitInterface::new(
        Generic16BitBus::new((
            gpiod.pd14.into_push_pull_output(),
            gpiod.pd15.into_push_pull_output(),
            gpiod.pd0.into_push_pull_output(),
            gpiod.pd1.into_push_pull_output(),
            gpioe.pe7.into_push_pull_output(),
            gpioe.pe8.into_push_pull_output(),
            gpioe.pe9.into_push_pull_output(),
            gpioe.pe10.into_push_pull_output(),
            gpioe.pe11.into_push_pull_output(),
            gpioe.pe12.into_push_pull_output(),
            gpioe.pe13.into_push_pull_output(),
            gpioe.pe14.into_push_pull_output(),
            gpioe.pe15.into_push_pull_output(),
            gpiod.pd8.into_push_pull_output(),
            gpiod.pd9.into_push_pull_output(),
            gpiod.pd10.into_push_pull_output(),
        )),
        gpiod.pd13.into_push_pull_output(),
        gpiod.pd5.into_push_pull_output(),
    );
    //CS
    gpiod.pd7.into_push_pull_output().set_low();
    //RD
    gpiod.pd4.into_push_pull_output().set_high();
```

```sh
controller
write command 0 0x1 // software reset
write command 0 0x36 // memory access control
write command 0 0x3a // pixel format
write command 0 0x11 // sleep oup
write command 0 0x29 // display on
write command 0 0x2a // column address space
write command 0 0x2b // page address set
write command 0 0x2c // memory write
done text

//alt
write command 0 0x1
write command 0 0x36
write command 1 0x28 (40) 0b00101000
write command 0 0x3a
write command 1 0x55 (85) 0b01010101
write command 0 0x11
write command 0 0x29
loop
write command 0 0x2a
write command 1 0x00 (0) 0b00000000
write command 2 0x00 (0) 0b00000000
write command 3 0x01 (1) 0b00000001
write command 4 0x40 (64) 0b01000000
write command 0 0x2b
write command 1 0x00 (0) 0b00000000
write command 2 0x00 (0) 0b00000000
write command 3 0x00 (0) 0b00000000
write command 4 0xf0 (240) 0b11110000
write command 0 0x2c


//main updated
write command 0 0x1
write command 0 0x36
write command 1 0x28 (40) 0b00101000
write command 0 0x3a
write command 1 0x55 (85) 0b01010101
write command 0 0x11
write command 0 0x29
write command 0 0x2a
write command 1 0x00 (0) 0b00000000
write command 2 0x00 (0) 0b00000000
write command 3 0x01 (1) 0b00000001
write command 4 0x40 (64) 0b01000000
write command 0 0x2b
write command 1 0x00 (0) 0b00000000
write command 2 0x00 (0) 0b00000000
write command 3 0x00 (0) 0b00000000
write command 4 0xf0 (240) 0b11110000
```

