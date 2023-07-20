[![crates.io](https://img.shields.io/crates/v/lcd-i2c-backpack.svg)](https://crates.io/crates/lcd-i2c-backpack)
[![crates.io](https://img.shields.io/crates/d/lcd-i2c-backpack.svg)](https://crates.io/crates/lcd-i2c-backpack)

# lcd-i2c-backpack

This crate provides a [`Hardware`][lcd::Hardware] implementation for the
[`lcd`](https://docs.rs/lcd/latest/lcd/index.html) crate for the readily available I2C expander
"backpack" kits based on the [NXP PCF8574/PCF8574A IC](https://www.nxp.com/docs/en/data-sheet/PCF8574_PCF8574A.pdf).
This uses [the 1.0 alpha of `embedded-hal`](https://docs.rs/embedded-hal/1.0.0-alpha.11/embedded_hal/index.html)
to communicate with the board.

**This is _not_ the crate you want if your board is running Linux!** If you are running Linux (e.g. Armbian on a Raspberry Pi), you likely want the [lcd-pcf8574](https://docs.rs/lcd-pcf8574/latest/lcd_pcf8574/) crate instead.

![I2C LCD Backpack Board](https://kanga.org/dacut/Pictures/I2C-LCD-Backpack-300x300.jpg)  
I2C LCD Backpack board from [PMD Way](https://pmdway.com/products/i2c-backpack-for-hd44780-compatible-lcd-modules-50-pack)

### Note on pins
The default pin configuration was determined from various schematics found on the Internet and verified by
probing pin connections on the board I have (whose actual manufacturer is unknown).

This is _not_ (yet) compatible with the [Adafruit I2C/SPI Backpack](https://learn.adafruit.com/i2c-spi-lcd-backpack);
the IC used requires a more complex communication protocol over I2C. (The pin assignment is also different.)

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
