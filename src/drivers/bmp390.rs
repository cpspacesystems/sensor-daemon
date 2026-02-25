//! Driver code for interaction with the BMP390 altemeter/barometer/thermometer.

use crate::i2c;
use std::io;

/// Constant Chip ID of the BMP390.
const CHIP_ID: u8 = 0x60;

const ADDRESS_DEFAULT: i2c::Address = i2c::Address::SevenBit(0b1110110);
const ADDRESS_SDO_VDDIO: i2c::Address = i2c::Address::SevenBit(0b1110110 | 0b0000001);

/// The three [`Register`]s which make up the pressure reading. This constant may be given to
/// [`read_data`] to get pressure data.
///
/// [`Register`]: Register
/// [`read_data`]: read_data
pub const REGS_PRESSURE: [Register; 3] = [Register::Data0, Register::Data1, Register::Data2];

/// The three [`Register`]s which make up the tempurature reading. This constant may be given to
/// [`read_data`] to get tempurature data.
///
/// [`Register`]: Register
/// [`read_data`]: read_data
pub const REGS_TEMPURATURE: [Register; 3] = [Register::Data3, Register::Data4, Register::Data5];

/// The three [`Register`]s which make up the sensor time on the BMP390. This constant may be given
/// to [`read_data`] to get the sensor time.
///
/// [`Register`]: Register
/// [`read_data`]: read_data
pub const REGS_SENSOR_TIME: [Register; 3] = [
    Register::SensorTime0,
    Register::SensorTime1,
    Register::SensorTime2,
];

/// Registers which are available on the BMP390.
#[repr(u8)]
#[derive(Eq, PartialEq, PartialOrd, Ord, Debug, Clone, Copy)]
pub enum Register {
    ChipId = 0x00,
    RevId = 0x01,
    ErrReg = 0x02,
    Status = 0x03,

    /// Pressure data bits 7 through 0
    Data0 = 0x04,
    /// Pressure data bits 15 through 8
    Data1 = 0x05,
    /// Pressure data bits 23 through 16
    Data2 = 0x06,

    /// Tempurature data bits 7 through 0
    Data3 = 0x07,
    /// Tempurature data bits 15 through 8
    Data4 = 0x08,
    /// Tempurature data bits 23 through 16
    Data5 = 0x09,

    /// Sensor time bits 7 through 0
    SensorTime0 = 0x0C,
    /// Sensor time bits 15 through 8
    SensorTime1 = 0x0D,
    /// Sensor time bits 23 through 16
    SensorTime2 = 0x0E,

    Event = 0x10,
    IntStatus = 0x11,

    /// FIFO byte counter bits 11 through 8
    FifoLength0 = 0x12,
    /// FIFO byte counter bits 7 through 0
    FifoLength1 = 0x13,

    FifoData = 0x14,

    /// Bits 7 though 0 of the FIFO watermark.
    FifoWtm0 = 0x15,
    /// Bit 8 (the LSB of this register) of the FIFO watermark.
    FifoWtm1 = 0x16,

    FifoConfig1 = 0x17,
    FifoConfig2 = 0x18,

    IntCtrl = 0x19,

    IfConf = 0x1A,

    PwrCtrl = 0x1B,

    /// Oversampling settings.
    Osr = 0x1C,
    /// Subdivision factor.
    Odr = 0x1D,

    Config = 0x1F,

    Cmd = 0x7E,
    // Calibration data occupies registers 0x30 through 0x57.
}

/// Set up a BMP390 on the given [`i2c::Bus`].
///
/// [`i2c::Bus`]: i2c::Bus
pub fn setup(bus: &mut i2c::Bus, sdo_vddio: bool) -> io::Result<i2c::Address> {
    let addr = match sdo_vddio {
        true => ADDRESS_DEFAULT,
        false => ADDRESS_SDO_VDDIO,
    };

    let mut buf = [0u8];
    bus.read_register(addr, Register::ChipId as _, &mut buf)?;

    if buf[0] != CHIP_ID {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "BMP390 `CHIP_ID` gave unexpected value",
        ));
    }

    Ok(addr)
}

/// Read a single eight bit [`Register`] and return its value.
///
/// [`Register`]: Register
pub fn read_register(bus: &mut i2c::Bus, addr: i2c::Address, reg: Register) -> io::Result<u8> {
    let mut buf = [0u8; 1];
    bus.read_register(addr, reg as _, &mut buf)?;
    Ok(buf[0])
}

/// Read out a three-register 24 bit piece of integer data. The given [`Register`] array holds the
/// three [`Register`]'s to read from and combine, the first element is the first and lowest eight
/// bits of the returned value, the second will occupy the next eight bits, between bits eight and
/// fifteen, so on and so forth.
///
/// Some constants are provided in this module which can fill the [`Register`] array argument.
///
/// [`Register`]: Register
pub fn read_data(bus: &mut i2c::Bus, addr: i2c::Address, regs: [Register; 3]) -> io::Result<u32> {
    let msb = read_register(bus, addr, regs[2])? as u32;
    let lsb = read_register(bus, addr, regs[1])? as u32;
    let xlsb = read_register(bus, addr, regs[0])? as u32;

    Ok((msb << 16) | (lsb << 8) | xlsb)
}

pub fn pressure_to_altitude(pressure: f32) -> f32 {
    const SEA_LEVEL: f32 = 0.0;

    let atmospheric = pressure / 100.0;
    44330.0 * (1.0 - (atmospheric / SEA_LEVEL).powf(0.1903))
}
