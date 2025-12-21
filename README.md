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

