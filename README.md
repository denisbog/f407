### flash on chip

```
cargo flash --release --chip STM32F407VGTx
```

update following lines in `.cargo/config.toml` to be able use cargo run command:

```
[target.'cfg(all(target_arch = "arm", target_os = "none"))']
runner = "probe-run --chip STM32F407VGTx"
```

