//! Module for working with the ICM 20948 gyroscope/accelerometer.

use crate::i2c;
use std::{
    io, mem,
    sync::{Arc, RwLock},
    thread,
    time::Duration
};

/// A more user friendly wrapper over an ICM20948.
#[derive(Debug)]
pub struct Icm20948 {
    bus: Arc<RwLock<i2c::Bus>>,
    address: Address,

    accel_scale: f32,
    gyro_scale: f32,
}

/// A frame of sensor data from an [`Icm20948`].
///
/// [`Icm20948`]: Icm20948
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct Icm20948Frame {
    /// Rotational velocity about the X-axis in degrees per second.
    pub gyro_x: f32,
    /// Rotational velocity about the Y-axis in degrees per second.
    pub gyro_y: f32,
    /// Rotational velocity about the Z-axis in degrees per second.
    pub gyro_z: f32,

    /// Acceleration along the X-axis in multiples of gravity.
    pub accel_x: f32,
    /// Acceleration along the Y-axis in multiples of gravity.
    pub accel_y: f32,
    /// Acceleration along the Z-axis in multiples of gravity.
    pub accel_z: f32,
}

/// The scale/range of accelerometer readings.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccelerometerScale {
    /// ± 2g
    Scale2G = 0,
    /// ± 4g
    Scale4G = 1,
    /// ± 8g
    Scale8G = 2,
    /// ± 16g
    Scale16G = 3,
}

/// The scale/range of gyro readings.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GyroScale {
    /// ± 250 degrees per second
    Scale250DegreesPerSecond = 0,
    /// ± 500 degrees per second
    Scale500DegreesPerSecond = 1,
    /// ± 1000 degrees per second
    Scale1000DegreesPerSecond = 2,
    /// ± 2000 degrees per second
    Scale2000DegreesPerSecond = 3,
}

impl Icm20948 {
    /// Open an [`Icm20948`] on the given [`i2c::Bus`].
    ///
    /// [`Icm20948`]: Icm20948
    /// [`i2c::Bus`]: i2c::Bus
    pub fn open(bus: Arc<RwLock<i2c::Bus>>, ad0_high: bool) -> io::Result<Icm20948> {
        let mut wr_bus = bus.write().expect("Lock should never be poisoned");
        let address = setup(&mut wr_bus, ad0_high)?;

        // Select bank 0, any function which use other banks should restore to bank 0 afterwards.
        write_register(
            &mut wr_bus,
            address,
            Register::Universal(UniversalRegister::RegBankSel),
            RegisterBankId::Bank0 as _,
        )?;

        write_register(
            &mut wr_bus,
            address,
            Register::UserBank0(Bank0Register::IntPinCfg),
            0b0000_0010,
        )?;

        mem::drop(wr_bus);

        Ok(Icm20948 {
            bus,
            address,
            accel_scale: 2f32,
            gyro_scale: 250f32,
        })
    }

    /// Method chaining version of [`Icm20948::set_accelerometer_scale`].
    ///
    /// [`Icm20948::set_accelerometer_scale`]: Icm20948::set_accelerometer_scale
    pub fn with_accelerometer_scale(mut self, scale: AccelerometerScale) -> io::Result<Self> {
        self.set_accelerometer_scale(scale)?;
        Ok(self)
    }

    /// Method chaining version of [`Icm20948::set_gyro_scale`].
    ///
    /// [`Icm20948::set_gyro_scale`]: Icm20948::set_gyro_scale
    pub fn with_gyro_scale(mut self, scale: GyroScale) -> io::Result<Self> {
        self.set_gyro_scale(scale)?;
        Ok(self)
    }

    /// Read a [`Icm20948Frame`] from the [`Icm20948`].
    ///
    /// [`Icm20948`]: Icm20948
    /// [`Icm20948Frame`]: Icm20948Frame
    pub fn read(&mut self) -> io::Result<Icm20948Frame> {
        Ok(Icm20948Frame {
            gyro_x: self.read_gyro_x()?,
            gyro_y: self.read_gyro_y()?,
            gyro_z: self.read_gyro_z()?,
            accel_x: self.read_accel_x()?,
            accel_y: self.read_accel_y()?,
            accel_z: self.read_accel_z()?,
        })
    }

    /// Read the acceleration in multiples of gravity.
    pub fn read_accel_x(&mut self) -> io::Result<f32> {
        let mut wr_bus = self.bus.write().expect("Lock should never be poisoned");

        let raw_accel = read_register_word(
            &mut wr_bus,
            self.address,
            Register::UserBank0(Bank0Register::AccelXoutH),
            Register::UserBank0(Bank0Register::AccelXoutL),
        )?;

        Ok((u16::cast_signed(raw_accel) as f32) / self.accel_scale)
    }

    /// Read the acceleration in multiples of gravity.
    pub fn read_accel_y(&mut self) -> io::Result<f32> {
        let mut wr_bus = self.bus.write().expect("Lock should never be poisoned");

        let raw_accel = read_register_word(
            &mut wr_bus,
            self.address,
            Register::UserBank0(Bank0Register::AccelYoutH),
            Register::UserBank0(Bank0Register::AccelYoutL),
        )?;

        Ok((u16::cast_signed(raw_accel) as f32) / self.accel_scale)
    }

    /// Read the acceleration in multiples of gravity.
    pub fn read_accel_z(&mut self) -> io::Result<f32> {
        let mut wr_bus = self.bus.write().expect("Lock should never be poisoned");

        let raw_accel = read_register_word(
            &mut wr_bus,
            self.address,
            Register::UserBank0(Bank0Register::AccelZoutH),
            Register::UserBank0(Bank0Register::AccelZoutL),
        )?;

        Ok((u16::cast_signed(raw_accel) as f32) / self.accel_scale)
    }

    /// Rate of rotation around the X axis in degrees per second.
    pub fn read_gyro_x(&mut self) -> io::Result<f32> {
        let mut wr_bus = self.bus.write().expect("Lock should never be poisoned");

        let raw_gyro = read_register_word(
            &mut wr_bus,
            self.address,
            Register::UserBank0(Bank0Register::GyroXoutH),
            Register::UserBank0(Bank0Register::GyroXoutL),
        )?;

        Ok((u16::cast_signed(raw_gyro) as f32) / self.gyro_scale)
    }

    /// Rate of rotation around the Y axis in degrees per second.
    pub fn read_gyro_y(&mut self) -> io::Result<f32> {
        let mut wr_bus = self.bus.write().expect("Lock should never be poisoned");

        let raw_gyro = read_register_word(
            &mut wr_bus,
            self.address,
            Register::UserBank0(Bank0Register::GyroYoutH),
            Register::UserBank0(Bank0Register::GyroYoutL),
        )?;

        Ok((u16::cast_signed(raw_gyro) as f32) / self.gyro_scale)
    }

    /// Rate of rotation around the Z axis in degrees per second.
    pub fn read_gyro_z(&mut self) -> io::Result<f32> {
        let mut wr_bus = self.bus.write().expect("Lock should never be poisoned");

        let raw_gyro = read_register_word(
            &mut wr_bus,
            self.address,
            Register::UserBank0(Bank0Register::GyroZoutH),
            Register::UserBank0(Bank0Register::GyroZoutL),
        )?;

        Ok((u16::cast_signed(raw_gyro) as f32) / self.gyro_scale)
    }

    /// Set the scale with which we measure accelerations. Our precision is inversely proportional
    /// to this.
    pub fn set_accelerometer_scale(&mut self, scale: AccelerometerScale) -> io::Result<()> {
        let data = 0x01 | ((scale as u8) << 1);
        let mut wr_bus = self.bus.write().expect("Lock should never be poisoned");

        write_register(
            &mut wr_bus,
            self.address,
            Register::Universal(UniversalRegister::RegBankSel),
            RegisterBankId::Bank2 as _,
        )?;

        let sel = read_register(
            &mut wr_bus,
            self.address,
            Register::Universal(UniversalRegister::RegBankSel),
        )?;

        if sel & 0b00_11_0000 != RegisterBankId::Bank2 as u8 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "ICM 20948 `REG_BANK_SEL` gave unexpected value",
            ));
        }

        write_register(
            &mut wr_bus,
            self.address,
            Register::UserBank2(Bank2Register::AccelConfig),
            data,
        )?;

        write_register(
            &mut wr_bus,
            self.address,
            Register::Universal(UniversalRegister::RegBankSel),
            RegisterBankId::Bank0 as _,
        )?;

        let sel = read_register(
            &mut wr_bus,
            self.address,
            Register::Universal(UniversalRegister::RegBankSel),
        )?;

        if sel & 0b00_11_0000 != RegisterBankId::Bank0 as u8 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "ICM 20948 `REG_BANK_SEL` gave unexpected value",
            ));
        }

        self.accel_scale = match scale {
            AccelerometerScale::Scale2G => 2f32,
            AccelerometerScale::Scale4G => 4f32,
            AccelerometerScale::Scale8G => 4f32,
            AccelerometerScale::Scale16G => 16f32,
        };

        Ok(())
    }

    /// Set the scale with which we measure angles. Our precision is inversely proportional to this.
    pub fn set_gyro_scale(&mut self, scale: GyroScale) -> io::Result<()> {
        let data = 0x01 | ((scale as u8) << 1);
        let mut wr_bus = self.bus.write().expect("Lock should never be poisoned");

        write_register(
            &mut wr_bus,
            self.address,
            Register::Universal(UniversalRegister::RegBankSel),
            RegisterBankId::Bank2 as _,
        )?;

        let sel = read_register(
            &mut wr_bus,
            self.address,
            Register::Universal(UniversalRegister::RegBankSel),
        )?;

        if sel & 0b00_11_0000 != RegisterBankId::Bank2 as u8 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "ICM 20948 `REG_BANK_SEL` gave unexpected value",
            ));
        }

        write_register(
            &mut wr_bus,
            self.address,
            Register::UserBank2(Bank2Register::GyroConfig1),
            data,
        )?;

        write_register(
            &mut wr_bus,
            self.address,
            Register::Universal(UniversalRegister::RegBankSel),
            RegisterBankId::Bank0 as _,
        )?;

        let sel = read_register(
            &mut wr_bus,
            self.address,
            Register::Universal(UniversalRegister::RegBankSel),
        )?;

        if sel & 0b00_11_0000 != RegisterBankId::Bank0 as u8 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "ICM 20948 `REG_BANK_SEL` gave unexpected value",
            ));
        }

        self.gyro_scale = match scale {
            GyroScale::Scale250DegreesPerSecond => 250f32,
            GyroScale::Scale500DegreesPerSecond => 500f32,
            GyroScale::Scale1000DegreesPerSecond => 1000f32,
            GyroScale::Scale2000DegreesPerSecond => 2000f32,
        };

        Ok(())
    }
}

/// The expected response of the ICM 20948 when requesting the `WHO_AM_I`
/// ([`Bank0Register::WhoAmI`]) register.
///
/// [`Bank0Register::WhoAmI`]: Bank0Register::WhoAmI
const WHO_AM_I: u8 = 0xEA;

/// The address of an ICM 20948.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Address {
    /// The default address with no modifications.
    #[default]
    DefaultAddress = 0b1101000,

    /// The address of the ICM 20948 when the AD0 pin is set high.
    Ad0HighAddress = Address::DefaultAddress as u8 | 0b0000001,
}

/// All available registers on the ICM 20948. Not all registers are available at all times however,
/// as each belongs to one of four banks which can be switched between.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Register {
    Universal(UniversalRegister),
    UserBank0(Bank0Register),
    UserBank1(Bank1Register),
    UserBank2(Bank2Register),
    UserBank3(Bank3Register),
}

#[repr(u8)]
enum RegisterBankId {
    Bank0 = 0b00_00_0000,
    Bank1 = 0b00_01_0000,
    Bank2 = 0b00_10_0000,
    Bank3 = 0b00_11_0000,
}

/// ICM 20948 registers where are available in all user banks.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum UniversalRegister {
    RegBankSel = 0x7F,
}

/// Registers which are available when the ICM 20948 is set to use user bank 0.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Bank0Register {
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
enum Bank1Register {
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
enum Bank2Register {
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
enum Bank3Register {
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
fn setup(bus: &mut i2c::Bus, ad0_high: bool) -> io::Result<Address> {
    let addr = match ad0_high {
        true => Address::Ad0HighAddress,
        false => Address::DefaultAddress,
    };

    bus.write(
        addr.into(),
        &mut [
            UniversalRegister::RegBankSel as _,
            RegisterBankId::Bank0 as _,
        ],
    )?;

    let mut buf = [0u8];
    bus.read_register(addr.into(), Bank0Register::WhoAmI as _, &mut buf)?;

    if buf[0] != WHO_AM_I {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "ICM 20948 `WHO_AM_I` gave unexpected value",
        ));
    }

    bus.read_register(addr.into(), UniversalRegister::RegBankSel as _, &mut buf)?;

    if buf[0] & 0b00_11_0000 != RegisterBankId::Bank0 as u8 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "ICM 20948 `REG_BANK_SEL` gave unexpected value",
        ));
    }

    bus.write(addr.into(), &mut [Bank0Register::PwrMgmt1 as _, 0x81])?;

    // Wait for chip to restart
    thread::sleep(Duration::from_millis(50));

    bus.write(addr.into(), &mut [Bank0Register::PwrMgmt1 as _, 0x01])?;

    bus.write(addr.into(), &mut [Bank0Register::PwrMgmt2 as _, 0])?;

    Ok(addr)
}

/// Read the given [`Register`] from an ICM 20948 at the given [`i2c::Address`]. This function will
/// not select the user bank for you, make sure to select the appropriate user bank before calling
/// this function, or otherwise ensure that the ICM20948 has the correct bank selected.
///
/// [`Register`]: Register
/// [`i2c::Address`]: i2c::Address
fn read_register(bus: &mut i2c::Bus, addr: Address, reg: Register) -> io::Result<u8> {
    let mut buf = [0u8; 1];
    bus.read_register(addr.into(), reg.into(), &mut buf)?;
    Ok(buf[0])
}

/// Read out two [`Register`]s from the ICM 20948 and, presuming that they represent the upper and
/// lower bytes of a 16-bit word, combine them into a single `u16` output.
///
/// [`Register`]: Register
fn read_register_word(
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
fn write_register(bus: &mut i2c::Bus, addr: Address, reg: Register, value: u8) -> io::Result<()> {
    bus.write(addr.into(), &mut [reg.into(), value])?;
    Ok(())
}

/// Write to a set of two [`Register`]s on the ICM 20948, presuming that they collectively represent
/// a single value, by splitting the given `value` into its least and most significant byte and
/// writing those to the `lower` and `upper` [`Register`]s respectively.
///
/// [`Register`]: Register
fn write_register_word(
    bus: &mut i2c::Bus,
    addr: Address,
    upper: Register,
    lower: Register,
    value: u16,
) -> io::Result<()> {
    let lsb = (value & 0x00FF) as u8;
    let msb = ((value & 0xFF00) >> 8) as u8;
    bus.write(addr.into(), &mut [lower.into(), lsb])?;
    bus.write(addr.into(), &mut [upper.into(), msb])?;
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
