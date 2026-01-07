//! Module for working with the ICM 20948 gyroscope/accelerometer.

use crate::i2c;
use std::io;

/// The expected response of the ICM 20948 when requesting the `WHO_AM_I`
/// ([`Bank0Register::WhoAmI`]) register.
///
/// [`Bank0Register::WhoAmI`]: Bank0Register::WhoAmI
const WHO_AM_I: u8 = 0xEA;

/// The address of an ICM 20948.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Address {
    /// The default address with no modifications.
    #[default]
    DefaultAddress = 0b1101000,

    /// The address of the ICM 20948 when the AD0 pin is set high.
    Ad0HighAddress = Address::DefaultAddress as u8 | 0b0000001,
}

/// All available registers on the ICM 20948. Not all registers are available at all times however,
/// as each belongs to one of four banks which can be switched between.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Register {
    Universal(UniversalRegister),
    UserBank0(Bank0Register),
    UserBank1(Bank1Register),
    UserBank2(Bank2Register),
    UserBank3(Bank3Register),
}

/// ICM 20948 registers where are available in all user banks.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum UniversalRegister {
    RegBankSel = 0x7F,
}

/// Registers which are available when the ICM 20948 is set to use user bank 0.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Bank0Register {
    WhoAmI = 0x00,

    UserCtrl = 0x03,

    LpConfig = 0x05,

    PwrMgmt1 = 0x06,
    PwrMgmt2 = 0x07,

    IntPinCfg = 0x0F,

    IntEnable = 0x10,

    IntEnable1 = 0x11,
    IntEnable2 = 0x12,
    IntEnable3 = 0x13,

    I2cMstStatus = 0x17,

    IntStatus = 0x19,

    IntStatus1 = 0x1A,
    IntStatus2 = 0x1B,
    IntStatus3 = 0x1C,

    DelayTimeH = 0x28,
    DelayTimeL = 0x29,

    AccelXoutH = 0x2D,
    AccelXoutL = 0x2E,
    AccelYoutH = 0x2F,
    AccelYoutL = 0x30,
    AccelZoutH = 0x31,
    AccelZoutL = 0x32,

    GyroXoutH = 0x33,
    GyroXoutL = 0x34,
    GyroYoutH = 0x35,
    GyroYoutL = 0x36,
    GyroZoutH = 0x37,
    GyroZoutL = 0x38,

    TempOutH = 0x39,
    TempOutL = 0x3A,

    ExtSlvSensData00 = 0x3B,
    ExtSlvSensData01 = 0x3C,
    ExtSlvSensData02 = 0x3D,
    ExtSlvSensData03 = 0x3E,
    ExtSlvSensData04 = 0x3F,
    ExtSlvSensData05 = 0x40,
    ExtSlvSensData06 = 0x41,
    ExtSlvSensData07 = 0x42,
    ExtSlvSensData08 = 0x43,
    ExtSlvSensData09 = 0x44,
    ExtSlvSensData10 = 0x45,
    ExtSlvSensData11 = 0x46,
    ExtSlvSensData12 = 0x47,
    ExtSlvSensData13 = 0x48,
    ExtSlvSensData14 = 0x49,
    ExtSlvSensData15 = 0x4A,
    ExtSlvSensData16 = 0x4B,
    ExtSlvSensData17 = 0x4C,
    ExtSlvSensData18 = 0x4D,
    ExtSlvSensData19 = 0x4E,
    ExtSlvSensData20 = 0x4F,
    ExtSlvSensData21 = 0x50,
    ExtSlvSensData22 = 0x51,
    ExtSlvSensData23 = 0x52,

    FifoEn1 = 0x66,
    FifoEn2 = 0x67,
    FifoRst = 0x68,
    FifoMode = 0x69,

    FifoCountH = 0x70,
    FifoCountL = 0x71,
    FifoRW = 0x72,

    DataRdyStatus = 0x74,

    FifoCfg = 0x76,
}

/// Registers which are available when the ICM 20948 is set to use user bank 1.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Bank1Register {
    SelfTestXGyro = 0x02,
    SelfTestYGyro = 0x03,
    SelfTestZGyro = 0x04,

    SelfTestXAccel = 0x0E,
    SelfTestYAccel = 0x0F,
    SelfTestZAccel = 0x10,

    XAOffsH = 0x14,
    XAOffsL = 0x15,

    YAOffsH = 0x17,
    YAOffsL = 0x18,

    ZAOffsH = 0x1A,
    ZAOffsL = 0x1B,

    TimebaseCorrectionPll = 0x28,
}

/// Registers which are available when the ICM 20948 is set to use user bank 2.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Bank2Register {
    GyroSmplrtDiv = 0x00,

    GyroConfig1 = 0x01,
    GyroConfig2 = 0x02,

    XGOffsUsrH = 0x03,
    XGOffsUsrL = 0x04,

    YGOffsUsrH = 0x05,
    YGOffsUsrL = 0x06,

    ZGOffsUsrH = 0x07,
    ZGOffsUsrL = 0x08,

    OdrAlignEn = 0x09,

    AccelSmplrtDiv1 = 0x10,
    AccelSmplrtDiv2 = 0x11,

    IntelCtrl = 0x12,

    AccelWomThr = 0x13,
    AccelConfig = 0x14,
    AccelConfig2 = 0x15,

    FsyncConfig = 0x52,
    TempConfig = 0x53,

    ModCtrlUsr = 0x54,
}

/// Registers which are available when the ICM 20948 is set to use user bank 3.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Bank3Register {
    I2cMstOdrConfig = 0x00,
    I2cMstCtrl = 0x01,
    I2cMstDelayCtrl = 0x02,

    I2cSlv0Addr = 0x03,
    I2cSlv0Reg = 0x04,
    I2cSlv0Ctrl = 0x05,
    I2cSlv0Do = 0x06,

    I2cSlv1Addr = 0x07,
    I2cSlv1Reg = 0x08,
    I2cSlv1Ctrl = 0x09,
    I2cSlv1Do = 0x0A,

    I2cSlv2Addr = 0x0B,
    I2cSlv2Reg = 0x0C,
    I2cSlv2Ctrl = 0x0D,
    I2cSlv2Do = 0x0E,

    I2cSlv3Addr = 0x0F,
    I2cSlv3Reg = 0x10,
    I2cSlv3Ctrl = 0x11,
    I2cSlv3Do = 0x12,

    I2cSlv4Addr = 0x13,
    I2cSlv4Reg = 0x14,
    I2cSlv4Ctrl = 0x15,
    I2cSlv4Do = 0x16,
    I2cSlv4Di = 0x17,
}

/// Set up an ICM 20948, returning the appropriate [`Address`] for the device given the `od0_high`
/// paramater, which should be `true` if the AD0 pin is set high on the device.
///
/// [`Address`]: Address
pub fn setup(bus: &mut i2c::Bus, ad0_high: bool) -> io::Result<Address> {
    let addr = match ad0_high {
        true => Address::Ad0HighAddress,
        false => Address::DefaultAddress,
    };

    bus.write(addr.into(), &[UniversalRegister::RegBankSel as _, 0])?;

    let mut buf = [0u8];
    bus.read_register(addr.into(), Bank0Register::WhoAmI as _, &mut buf)?;

    if buf[0] != WHO_AM_I {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "ICM 20948 `WHO_AM_I` gave unexpected value",
        ));
    }

    Ok(addr)
}

/// Read the given [`Register`] from an ICM 20948 at the given [`i2c::Address`]. This function will
/// not select the user bank for you, make sure to select the appropriate user bank before calling
/// this function, or otherwise ensure that the ICM20948 has the correct bank selected.
///
/// [`Register`]: Register
/// [`i2c::Address`]: i2c::Address
pub fn read_register(bus: &mut i2c::Bus, addr: Address, reg: Register) -> io::Result<u8> {
    let mut buf = [0u8; 1];
    bus.read_register(addr.into(), reg.into(), &mut buf)?;
    Ok(buf[0])
}

/// Read out two [`Register`]s from the ICM 20948 and, presuming that they represent the upper and
/// lower bytes of a 16-bit word, combine them into a single `u16` output.
///
/// [`Register`]: Register
pub fn read_register_word(
    bus: &mut i2c::Bus,
    addr: Address,
    upper: Register,
    lower: Register,
) -> io::Result<u16> {
    let msb = read_register(bus, addr.into(), upper.into())? as u16;
    let lsb = read_register(bus, addr.into(), lower.into())? as u16;
    Ok(msb << 8 | lsb)
}

/// Write to a [`Register`] on the ICM 20948.
///
/// [`Register`]: Register
pub fn write_register(
    bus: &mut i2c::Bus,
    addr: Address,
    reg: Register,
    value: u8,
) -> io::Result<()> {
    bus.write(addr.into(), &[reg.into(), value])?;
    Ok(())
}

/// Write to a set of two [`Register`]s on the ICM 20948, presuming that they collectively represent
/// a single value, by splitting the given `value` into its least and most significant byte and
/// writing those to the `lower` and `upper` [`Register`]s respectively.
///
/// [`Register`]: Register
pub fn write_register_word(
    bus: &mut i2c::Bus,
    addr: Address,
    upper: Register,
    lower: Register,
    value: u16,
) -> io::Result<()> {
    let lsb = (value & 0x00FF) as u8;
    let msb = ((value & 0xFF00) >> 8) as u8;
    bus.write(addr.into(), &[lower.into(), lsb])?;
    bus.write(addr.into(), &[upper.into(), msb])?;
    Ok(())
}

impl Into<u8> for Register {
    fn into(self) -> u8 {
        match self {
            Register::Universal(r) => r as _,
            Register::UserBank0(r) => r as _,
            Register::UserBank1(r) => r as _,
            Register::UserBank2(r) => r as _,
            Register::UserBank3(r) => r as _,
        }
    }
}

impl Into<i2c::Address> for Address {
    fn into(self) -> i2c::Address {
        i2c::Address::SevenBit(self as _)
    }
}
