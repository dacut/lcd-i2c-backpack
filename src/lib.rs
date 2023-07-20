//! This crate provides a [`Hardware`][lcd::Hardware] implementation for the
//! [`lcd`](https://docs.rs/lcd/latest/lcd/index.html) crate for the readily available I2C expander
//! "backpack" kits based on the [NXP PCF8574/PCF8574A IC](https://www.nxp.com/docs/en/data-sheet/PCF8574_PCF8574A.pdf).
//! This uses [the 1.0 alpha of `embedded-hal`](https://docs.rs/embedded-hal/1.0.0-alpha.11/embedded_hal/index.html)
//! to communicate with the board.
//!
//! **This is _not_ the crate you want if your board is running Linux!** If you are running Linux (e.g. Armbian on a Raspberry Pi), you likely want the [lcd-pcf8574](https://docs.rs/lcd-pcf8574/latest/lcd_pcf8574/) crate instead.
//!
//! ![I2C LCD Backpack Board](https://kanga.org/dacut/Pictures/I2C-LCD-Backpack-300x300.jpg)
//!
//!
//! I2C LCD Backpack board from [PMD Way](https://pmdway.com/products/i2c-backpack-for-hd44780-compatible-lcd-modules-50-pack)
//!
//! ## Note on pins
//! The default pin configuration was determined from various schematics found on the Internet and verified by
//! probing pin connections on the board I have (whose actual manufacturer is unknown).
//!
//! This is _not_ (yet) compatible with the [Adafruit I2C/SPI Backpack](https://learn.adafruit.com/i2c-spi-lcd-backpack);
//! the IC used requires a more complex communication protocol over I2C. (The pin assignment is also different.)
#![no_std]
#![warn(clippy::all)]
#![deny(rustdoc::missing_crate_level_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(missing_docs)]

use {
    core::fmt::{Debug, Formatter, Result as FmtResult},
    embedded_hal::i2c::{I2c, SevenBitAddress},
    lcd::{Backlight, FunctionMode, Hardware},
};

/// LCD character display hardware implementation for the I2C "backpack" kit and embedded-hal.
///
/// # Example for the ESP32
///
/// ```ignore
/// use esp_idf_hal::Peripherals;
/// use lcd::{Backlight, Display, DisplayBlink, DisplayCursor, DisplayMode, FunctionDots, FunctionLine};
/// use lcd_freertos_delay::FreeRtosDelay;
/// use lcd_i2c_backpack::I2cLcdBackpack;
///
/// let peripherals = Peripherals::take().unwrap();
/// let i2c0 = peripherals.i2c0;
/// let sda = peripherals.pins.gpio2;
/// let scl = peripherals.pins.gpio3;
/// let i2c_config = I2cConfig::new().baudrate(Hertz(100_000));
/// let i2c = I2cDriver::new(i2c0, sda, scl, &i2c_config).unwrap();
///
/// let hw = I2cLcdBackpack::new(i2c, ADDR);
/// let delay = FreeRtosDelay::new();
/// let mut display = Display::new(HardwareDelay::new(hw, delay));
///
/// display.init(FunctionLine::Line2, FunctionDots::Dots5x8);
/// display.display(DisplayMode::DisplayOn, DisplayCursor::CursorOn, DisplayBlink::BlinkOn);
/// display.set_backlight(true);
/// display.clear();
/// display.print("Hello world!");
/// ```
pub struct I2cLcdBackpack<T> {
    driver: T,
    address: u8,
    state: u8,
    pins: I2cLcdPinConfig,
}

const DEFAULT_RS_PIN: u8 = 0;
const DEFAULT_RW_PIN: u8 = 1;
const DEFAULT_EN_PIN: u8 = 2;
const DEFAULT_D4_PIN: u8 = 4;
const DEFAULT_D5_PIN: u8 = 5;
const DEFAULT_D6_PIN: u8 = 6;
const DEFAULT_D7_PIN: u8 = 7;
const DEFAULT_BACKLIGHT_PIN: u8 = 3;

const RW_PIN_NONE: u8 = 0xff;

impl<T> I2cLcdBackpack<T> {
    /// Create a new LCD I2C backpack hardware struct using the given I2C HAL driver communicating with
    /// a PCF8574 chip at the given address, using the default (common) pin assignment.
    pub fn new(driver: T, address: u8) -> Self {
        Self::new_with_pins(driver, address, I2cLcdPinConfig::default())
    }

    /// Create a new LCD I2C driver communicating using the given I2C HAL driver communicating with
    /// a PCF8574 chip at the given address, using a custom pin assignment.
    pub fn new_with_pins(driver: T, address: u8, pins: I2cLcdPinConfig) -> Self {
        Self {
            driver,
            address,
            state: 0,
            pins,
        }
    }
}

#[inline]
fn check_pin(pin: u8) {
    if pin > 7 {
        panic!("pins must be between 0 and 7");
    }
}

impl<T: I2c<SevenBitAddress>> Hardware for I2cLcdBackpack<T> {
    fn rs(&mut self, bit: bool) {
        if bit {
            self.state |= 1 << self.pins.rs_pin;
        } else {
            self.state &= !(1 << self.pins.rs_pin);
        }
    }

    fn enable(&mut self, bit: bool) {
        if bit {
            self.state |= 1 << self.pins.en_pin;
        } else {
            self.state &= !(1 << self.pins.en_pin);
        }
    }

    fn data(&mut self, data: u8) {
        if data & 0b0001 != 0 {
            self.state |= 1 << self.pins.d4_pin;
        } else {
            self.state &= !(1 << self.pins.d4_pin);
        }

        if data & 0b0010 != 0 {
            self.state |= 1 << self.pins.d5_pin;
        } else {
            self.state &= !(1 << self.pins.d5_pin);
        }

        if data & 0b0100 != 0 {
            self.state |= 1 << self.pins.d6_pin;
        } else {
            self.state &= !(1 << self.pins.d6_pin);
        }

        if data & 0b1000 != 0 {
            self.state |= 1 << self.pins.d7_pin;
        } else {
            self.state &= !(1 << self.pins.d7_pin);
        }
    }

    fn mode(&self) -> FunctionMode {
        FunctionMode::Bit4
    }

    fn can_read(&self) -> bool {
        self.pins.rw_pin != RW_PIN_NONE
    }

    fn rw(&mut self, bit: bool) {
        if self.pins.rw_pin == RW_PIN_NONE {
            panic!("cannot read from LCD");
        }

        if bit {
            // Configure all data pins as inputs.
            self.data(0b1111);
            self.state |= 1 << self.pins.rw_pin;
        } else {
            self.state &= !(1 << self.pins.rw_pin);
        }
    }

    fn read_data(&mut self) -> u8 {
        let mut result: [u8; 1] = [0; 1];
        self.driver.read(self.address, &mut result).unwrap();
        let result = result[0];

        let mut data = 0;
        if result & (1 << self.pins.d4_pin) != 0 {
            data |= 0b0001;
        }

        if result & (1 << self.pins.d5_pin) != 0 {
            data |= 0b0010;
        }

        if result & (1 << self.pins.d6_pin) != 0 {
            data |= 0b0100;
        }

        if result & (1 << self.pins.d7_pin) != 0 {
            data |= 0b1000;
        }

        data
    }

    fn apply(&mut self) {
        self.driver.write(self.address, &[self.state]).unwrap();
    }
}

impl<T: Debug> Debug for I2cLcdBackpack<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("I2cLcdBackpack")
            .field("driver", &self.driver)
            .field("address", &self.address)
            .field("state", &self.state)
            .field("pins", &self.pins)
            .finish()
    }
}

impl<T: I2c<SevenBitAddress>> Backlight for I2cLcdBackpack<T> {
    fn set_backlight(&mut self, enable: bool) {
        if enable {
            self.state |= 1 << self.pins.backlight_pin;
        } else {
            self.state &= !(1 << self.pins.backlight_pin);
        }

        self.apply();
    }
}

/// The pin assignment configuration for the I2C LCD backpack.
#[derive(Clone, Copy, Debug)]
pub struct I2cLcdPinConfig {
    rw_pin: u8,
    rs_pin: u8,
    en_pin: u8,
    d4_pin: u8,
    d5_pin: u8,
    d6_pin: u8,
    d7_pin: u8,
    backlight_pin: u8,
}

impl Default for I2cLcdPinConfig {
    fn default() -> Self {
        Self {
            rw_pin: DEFAULT_RW_PIN,
            rs_pin: DEFAULT_RS_PIN,
            en_pin: DEFAULT_EN_PIN,
            d4_pin: DEFAULT_D4_PIN,
            d5_pin: DEFAULT_D5_PIN,
            d6_pin: DEFAULT_D6_PIN,
            d7_pin: DEFAULT_D7_PIN,
            backlight_pin: DEFAULT_BACKLIGHT_PIN,
        }
    }
}

impl I2cLcdPinConfig {
    /// Set the read/write output from the PCF8574. If `None` is passed, reading from the LCD will be disabled.
    ///
    /// The default assignment is output 1.
    pub fn rw(mut self, rw_pin: Option<u8>) -> Self {
        self.rw_pin = match rw_pin {
            Some(rw_pin) => {
                check_pin(rw_pin);
                rw_pin
            }
            None => RW_PIN_NONE,
        };
        self
    }

    /// Set the register select (RS) output from the PCF8574.
    ///
    /// The default assignment is output 0.
    pub fn rs(mut self, rs_pin: u8) -> Self {
        check_pin(rs_pin);
        self.rs_pin = rs_pin;
        self
    }

    /// Set the enable (EN or E) output from the PCF8574.
    ///
    /// The default assignment is output 2.
    pub fn en(mut self, en_pin: u8) -> Self {
        check_pin(en_pin);
        self.en_pin = en_pin;
        self
    }

    /// Set the data 4 output from the PCF8574.
    ///
    /// The default assignment is output 4.
    pub fn d4(mut self, d4_pin: u8) -> Self {
        check_pin(d4_pin);
        self.d4_pin = d4_pin;
        self
    }

    /// Set the data 5 output from the PCF8574.
    ///
    /// The default assignment is output 5.
    pub fn d5(mut self, d5_pin: u8) -> Self {
        check_pin(d5_pin);
        self.d5_pin = d5_pin;
        self
    }

    /// Set the data 4 output from the PCF8574.
    ///
    /// The default assignment is output 6.
    pub fn d6(mut self, d6_pin: u8) -> Self {
        check_pin(d6_pin);
        self.d6_pin = d6_pin;
        self
    }

    /// Set the data 4 output from the PCF8574.
    ///
    /// The default assignment is output 7.
    pub fn d7(mut self, d7_pin: u8) -> Self {
        check_pin(d7_pin);
        self.d7_pin = d7_pin;
        self
    }

    /// Set the backlight enable output from the PCF8574.
    ///
    /// The default assignment is output 3.
    pub fn backlight(mut self, backlight_pin: u8) -> Self {
        check_pin(backlight_pin);
        self.backlight_pin = backlight_pin;
        self
    }
}
