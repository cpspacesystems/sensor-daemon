//! Driver code for interaction with the BMP390 altemeter/barometer/thermometer.

use crate::i2c;
use std::{
    io, mem,
    sync::{Arc, RwLock},
};

/// A more user-friendly wrapper over an BMP390.
pub struct Bmp390 {
    bus: Arc<RwLock<i2c::Bus>>,
    address: i2c::Address,
    calibration_data: FloatingCalibrationData,
}

/// A frame of data from the [`Bmp390`]. This contains the sensor data from the device.
///
/// [`Bmp390`]: Bmp390
pub struct Bmp390Frame {
    /// Tempurature in celsius.
    pub tempurature: f32,

    /// Pressure in pascals.
    pub pressure: f32,

    /// Altitude in meters.
    pub altitude: f32,
}

#[repr(u8)]
pub enum Oversampling {
    NoOversampling = 0b000,
    Oversample2x = 0b001,
    Oversample4x = 0b010,
    Oversample8x = 0b011,
    Oversample16x = 0b100,
    Oversample32x = 0b101,
}

impl Bmp390 {
    /// Open a BMP390 device on the I2C bus. This function will collect calibration data for future
    /// calls to methods off of the returned [`Bmp390`].
    ///
    /// [`Bmp390`]: Bmp390
    pub fn open(bus: Arc<RwLock<i2c::Bus>>, sdo_vddio: bool) -> io::Result<Bmp390> {
        let mut wr_bus = bus.write().expect("Lock should never be poisoned");
        let address = setup(&mut wr_bus, sdo_vddio)?;
        let raw_calibration_data = read_calibration(&mut wr_bus, address)?;
        let calibration_data: FloatingCalibrationData = raw_calibration_data.into();

        // `wr_bus` has a borrow of `bus`, which needs to drop before we can move `bus`
        mem::drop(wr_bus);

        Ok(Bmp390 {
            bus,
            address,
            calibration_data,
        })
    }

    /// Read a [`Bmp390Frame`] from the [`Bmp390`]. This is a data type holding all values which we
    /// seek to produce with a BMP390 sensor.
    ///
    /// [`Bmp390`]: Bmp390
    /// [`Bmp390Frame`]: Bmp390Frame
    pub fn read(&mut self) -> io::Result<Bmp390Frame> {
        let mut wr_bus = self.bus.write().expect("Lock should never be poisoned");

        // Order here is important, the tempurature compensation records a value which is used in
        // the pressure compensation.
        let raw_temp = read_data(&mut wr_bus, self.address, REGS_TEMPURATURE)?;
        let raw_pressure = read_data(&mut wr_bus, self.address, REGS_PRESSURE)?;

        mem::drop(wr_bus);

        let tempurature = compensate_tempurature(raw_temp, &mut self.calibration_data);
        let pressure = compensate_pressure(raw_pressure, &self.calibration_data);
        let altitude = pressure_to_altitude(pressure);

        Ok(Bmp390Frame {
            tempurature,
            pressure,
            altitude,
        })
    }

    /// Set the oversampling options for tempurature and pressure. It is recommended by the BMP390
    /// data sheet that for pressure oversampling x1 through x8 inclusive you use no oversampling
    /// for tempurature, and for pressure oversampling x16 and x32 you use x2 oversampling for
    /// tempurature.
    pub fn set_oversampling(
        &mut self,
        tempurature: Oversampling,
        pressure: Oversampling,
    ) -> io::Result<()> {
        let data = 0u8 | pressure as u8 | ((tempurature as u8) << 3);
        let mut wr_bus = self.bus.write().expect("Lock should never be poisoned");
        write_register(&mut wr_bus, self.address, Register::Osr, data)
    }
}

/// Constant Chip ID of the BMP390.
const CHIP_ID: u8 = 0x60;

const ADDRESS_DEFAULT: i2c::Address = i2c::Address::SevenBit(0b1110110);
const ADDRESS_SDO_VDDIO: i2c::Address = i2c::Address::SevenBit(0b1110110 | 0b0000001);

/// The three [`Register`]s which make up the pressure reading. This constant may be given to
/// [`read_data`] to get pressure data.
///
/// [`Register`]: Register
/// [`read_data`]: read_data
const REGS_PRESSURE: [Register; 3] = [Register::Data0, Register::Data1, Register::Data2];

/// The three [`Register`]s which make up the tempurature reading. This constant may be given to
/// [`read_data`] to get tempurature data.
///
/// [`Register`]: Register
/// [`read_data`]: read_data
const REGS_TEMPURATURE: [Register; 3] = [Register::Data3, Register::Data4, Register::Data5];

/// The three [`Register`]s which make up the sensor time on the BMP390. This constant may be given
/// to [`read_data`] to get the sensor time.
///
/// [`Register`]: Register
/// [`read_data`]: read_data
const REGS_SENSOR_TIME: [Register; 3] = [
    Register::SensorTime0,
    Register::SensorTime1,
    Register::SensorTime2,
];

/// Registers which are available on the BMP390.
#[repr(u8)]
#[derive(Eq, PartialEq, PartialOrd, Ord, Debug, Clone, Copy)]
enum Register {
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

    /// Oversampling settings. Bits 0 through 2 are for pressure oversampling, bits 3 through 5 are
    /// for tempurature oversampling.
    Osr = 0x1C,
    /// Subdivision factor.
    Odr = 0x1D,

    Config = 0x1F,

    /// Bits 7 through 0 of `NVM_PAR_T1`, `NVM_PAR_T1` is unsigned 16 bit.
    NvmParT1Lsb = 0x31,
    /// Bits 15 through 8 of `NVM_PAR_T1`, `NVM_PAR_T1` is unsigned 16 bit.
    NvmParT1Msb = 0x32,

    /// Bits 7 through 0 of `NVM_PAR_T2`, `NVM_PAR_T2` is unsigned 16 bit.
    NvmParT2Lsb = 0x33,
    /// Bits 15 through 8 of `NVM_PAR_T2`, `NVM_PAR_T2` is unsigned 16 bit.
    NvmParT2Msb = 0x34,

    /// `NVM_PAR_T3`, which is a signed 8 bit integer.
    NvmParT3 = 0x35,

    /// Bits 7 through 0 of `NVM_PAR_P1`, `NVM_PAR_P1` is signed 16 bit.
    NvmParP1Lsb = 0x36,
    /// Bits 15 through 8 of `NVM_PAR_P1`, `NVM_PAR_P1` is signed 16 bit.
    NvmParP1Msb = 0x37,

    /// Bits 7 through 0 of `NVM_PAR_P2`, `NVM_PAR_P2` is signed 16 bit.
    NvmParP2Lsb = 0x38,
    /// Bits 15 through 8 of `NVM_PAR_P2`, `NVM_PAR_P2` is signed 16 bit.
    NvmParP2Msb = 0x39,

    /// `NVM_PAR_P3`, which is a signed 8 bit integer.
    NvmParP3 = 0x3A,

    /// `NVM_PAR_P4`, which is a signed 8 bit integer.
    NvmParP4 = 0x3B,

    /// Bits 7 through 0 of `NVM_PAR_P5`, `NVM_PAR_P5` is unsigned 16 bit.
    NvmParP5Lsb = 0x3C,
    /// Bits 15 through 8 of `NVM_PAR_P5`, `NVM_PAR_P5` is unsigned 16 bit.
    NvmParP5Msb = 0x3D,

    /// Bits 7 through 0 of `NVM_PAR_P6`, `NVM_PAR_P6` is unsigned 16 bit.
    NvmParP6Lsb = 0x3E,
    /// Bits 15 through 8 of `NVM_PAR_P6`, `NVM_PAR_P6` is unsigned 16 bit.
    NvmParP6Msb = 0x3F,

    /// `NVM_PAR_P7`, which is a signed 8 bit integer.
    NvmParP7 = 0x40,

    /// `NVM_PAR_P8`, which is a signed 8 bit integer.
    NvmParP8 = 0x41,

    /// Bits 7 through 0 of `NVM_PAR_P9`, `NVM_PAR_P9` is signed 16 bit.
    NvmParP9Lsb = 0x42,
    /// Bits 15 through 8 of `NVM_PAR_P9`, `NVM_PAR_P9` is signed 16 bit.
    NvmParP9Msb = 0x43,

    /// `NVM_PAR_P11`, which is a signed 8 bit integer.
    NvmParP10 = 0x44,

    /// `NVM_PAR_P11`, which is a signed 8 bit integer.
    NvmParP11 = 0x45,

    Cmd = 0x7E,
}

/// The calibration constants which we can get from the BMP390. The data in this data type is not
/// directly usable, it should be converted first to a [`FloatingCalibrationData`].
///
/// [`FloatingCalibrationData`]: FloatingCalibrationData
#[derive(Clone, Copy, Debug)]
struct CalibrationData {
    nvm_par_t1: u16,
    nvm_par_t2: u16,
    nvm_par_t3: i8,

    nvm_par_p1: i16,
    nvm_par_p2: i16,
    nvm_par_p3: i8,
    nvm_par_p4: i8,
    nvm_par_p5: u16,
    nvm_par_p6: u16,
    nvm_par_p7: i8,
    nvm_par_p8: i8,
    nvm_par_p9: i16,
    nvm_par_p10: i8,
    nvm_par_p11: i8,
}

/// Directly usable calibration data, as converted from the integer constants from a BMP390.
#[derive(Clone, Copy, Debug)]
struct FloatingCalibrationData {
    par_t1: f32,
    par_t2: f32,
    par_t3: f32,

    par_p1: f32,
    par_p2: f32,
    par_p3: f32,
    par_p4: f32,
    par_p5: f32,
    par_p6: f32,
    par_p7: f32,
    par_p8: f32,
    par_p9: f32,
    par_p10: f32,
    par_p11: f32,

    /// Prior temp measurements which are used in compensating the pressure measurement.
    t_lin: f32,
}

/// Set up a BMP390 on the given [`i2c::Bus`].
///
/// [`i2c::Bus`]: i2c::Bus
fn setup(bus: &mut i2c::Bus, sdo_vddio: bool) -> io::Result<i2c::Address> {
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

/// Read calibration data off of the BMP390.
fn read_calibration(bus: &mut i2c::Bus, addr: i2c::Address) -> io::Result<CalibrationData> {
    let nvm_par_t1_lsb = read_register(bus, addr, Register::NvmParT1Lsb)?;
    let nvm_par_t1_msb = read_register(bus, addr, Register::NvmParT1Msb)?;
    let nvm_par_t1 = ((nvm_par_t1_msb as u16) << 8) | (nvm_par_t1_lsb as u16);

    let nvm_par_t2_lsb = read_register(bus, addr, Register::NvmParT2Lsb)?;
    let nvm_par_t2_msb = read_register(bus, addr, Register::NvmParT2Msb)?;
    let nvm_par_t2 = ((nvm_par_t2_msb as u16) << 8) | (nvm_par_t2_lsb as u16);

    let nvm_par_t3 = read_register(bus, addr, Register::NvmParT3)?;
    let nvm_par_t3: i8 = u8::cast_signed(nvm_par_t3);

    let nvm_par_p1_lsb = read_register(bus, addr, Register::NvmParP1Lsb)?;
    let nvm_par_p1_msb = read_register(bus, addr, Register::NvmParP1Msb)?;
    let nvm_par_p1 = ((nvm_par_p1_msb as i16) << 8) | (nvm_par_p1_lsb as i16);

    let nvm_par_p2_lsb = read_register(bus, addr, Register::NvmParP2Lsb)?;
    let nvm_par_p2_msb = read_register(bus, addr, Register::NvmParP2Msb)?;
    let nvm_par_p2 = ((nvm_par_p2_msb as i16) << 8) | (nvm_par_p2_lsb as i16);

    let nvm_par_p3 = read_register(bus, addr, Register::NvmParP3)?;
    let nvm_par_p3: i8 = u8::cast_signed(nvm_par_p3);

    let nvm_par_p4 = read_register(bus, addr, Register::NvmParP4)?;
    let nvm_par_p4: i8 = u8::cast_signed(nvm_par_p4);

    let nvm_par_p5_lsb = read_register(bus, addr, Register::NvmParP5Lsb)?;
    let nvm_par_p5_msb = read_register(bus, addr, Register::NvmParP5Msb)?;
    let nvm_par_p5 = ((nvm_par_p5_msb as u16) << 8) | (nvm_par_p5_lsb as u16);

    let nvm_par_p6_lsb = read_register(bus, addr, Register::NvmParP6Lsb)?;
    let nvm_par_p6_msb = read_register(bus, addr, Register::NvmParP6Msb)?;
    let nvm_par_p6 = ((nvm_par_p6_msb as u16) << 8) | (nvm_par_p6_lsb as u16);

    let nvm_par_p7 = read_register(bus, addr, Register::NvmParP7)?;
    let nvm_par_p7: i8 = u8::cast_signed(nvm_par_p7);

    let nvm_par_p8 = read_register(bus, addr, Register::NvmParP8)?;
    let nvm_par_p8: i8 = u8::cast_signed(nvm_par_p8);

    let nvm_par_p9_lsb = read_register(bus, addr, Register::NvmParP9Lsb)?;
    let nvm_par_p9_msb = read_register(bus, addr, Register::NvmParP9Msb)?;
    let nvm_par_p9 = ((nvm_par_p9_msb as i16) << 8) | (nvm_par_p9_lsb as i16);

    let nvm_par_p10 = read_register(bus, addr, Register::NvmParP10)?;
    let nvm_par_p10: i8 = u8::cast_signed(nvm_par_p10);

    let nvm_par_p11 = read_register(bus, addr, Register::NvmParP11)?;
    let nvm_par_p11: i8 = u8::cast_signed(nvm_par_p11);

    Ok(CalibrationData {
        nvm_par_t1,
        nvm_par_t2,
        nvm_par_t3,
        nvm_par_p1,
        nvm_par_p2,
        nvm_par_p3,
        nvm_par_p4,
        nvm_par_p5,
        nvm_par_p6,
        nvm_par_p7,
        nvm_par_p8,
        nvm_par_p9,
        nvm_par_p10,
        nvm_par_p11,
    })
}

/// Read a single eight bit [`Register`] and return its value.
///
/// [`Register`]: Register
fn read_register(bus: &mut i2c::Bus, addr: i2c::Address, reg: Register) -> io::Result<u8> {
    let mut buf = [0u8; 1];
    bus.read_register(addr, reg as _, &mut buf)?;
    Ok(buf[0])
}

/// Write a single eight bit [`Register`].
///
/// [`Register`]: Register
fn write_register(
    bus: &mut i2c::Bus,
    addr: i2c::Address,
    reg: Register,
    data: u8,
) -> io::Result<()> {
    let buf = [reg as u8, data];
    bus.write(addr, &buf)
}

/// Read out a three-register 24 bit piece of integer data. The given [`Register`] array holds the
/// three [`Register`]'s to read from and combine, the first element is the first and lowest eight
/// bits of the returned value, the second will occupy the next eight bits, between bits eight and
/// fifteen, so on and so forth.
///
/// Some constants are provided in this module which can fill the [`Register`] array argument.
///
/// [`Register`]: Register
fn read_data(bus: &mut i2c::Bus, addr: i2c::Address, regs: [Register; 3]) -> io::Result<u32> {
    let msb = read_register(bus, addr, regs[2])? as u32;
    let lsb = read_register(bus, addr, regs[1])? as u32;
    let xlsb = read_register(bus, addr, regs[0])? as u32;

    Ok((msb << 16) | (lsb << 8) | xlsb)
}

fn pressure_to_altitude(pressure: f32) -> f32 {
    const SEA_LEVEL: f32 = 1013.25;

    let atmospheric = pressure / 100.0;
    44330.0 * (1.0 - (atmospheric / SEA_LEVEL).powf(0.1903))
}

fn compensate_tempurature(measurement: u32, calibration_data: &mut FloatingCalibrationData) -> f32 {
    let partial_data_1 = measurement as f32 - calibration_data.par_t1;
    let partial_data_2 = measurement as f32 * calibration_data.par_t2;
    calibration_data.t_lin = partial_data_2 + partial_data_1.powi(2) * calibration_data.par_t3;
    calibration_data.t_lin
}

fn compensate_pressure(measurement: u32, calibration_data: &FloatingCalibrationData) -> f32 {
    let partial_data_1 = calibration_data.par_p6 * calibration_data.t_lin;
    let partial_data_2 = calibration_data.par_p7 * calibration_data.t_lin.powi(2);
    let partial_data_3 = calibration_data.par_p8 * calibration_data.t_lin.powi(3);

    let partial_out_1 = calibration_data.par_p5 + partial_data_1 + partial_data_2 + partial_data_3;

    let partial_data_1 = calibration_data.par_p2 * calibration_data.t_lin;
    let partial_data_2 = calibration_data.par_p3 * calibration_data.t_lin.powi(2);
    let partial_data_3 = calibration_data.par_p4 * calibration_data.t_lin.powi(3);

    let partial_out_2 = measurement as f32
        * (calibration_data.par_p1 + partial_data_1 + partial_data_2 + partial_data_3);

    let partial_data_1 = (measurement as f32).powi(2);
    let partial_data_2 =
        calibration_data.par_p9 + calibration_data.par_p10 * calibration_data.t_lin;
    let partial_data_3 = partial_data_1 * partial_data_2;

    let partial_out_3 = partial_data_3 + (measurement as f32).powi(3) * calibration_data.par_p11;

    partial_out_1 + partial_out_2 + partial_out_3
}

impl From<CalibrationData> for FloatingCalibrationData {
    fn from(v: CalibrationData) -> Self {
        FloatingCalibrationData {
            par_t1: v.nvm_par_t1 as f32 / 2f32.powi(-8),
            par_t2: v.nvm_par_t2 as f32 / 2f32.powi(30),
            par_t3: v.nvm_par_t3 as f32 / 2f32.powi(48),

            par_p1: v.nvm_par_p1 as f32 / 2f32.powi(20),
            par_p2: v.nvm_par_p2 as f32 / 2f32.powi(29),
            par_p3: v.nvm_par_p3 as f32 / 2f32.powi(32),
            par_p4: v.nvm_par_p4 as f32 / 2f32.powi(37),
            par_p5: v.nvm_par_p5 as f32 / 2f32.powi(-3),
            par_p6: v.nvm_par_p6 as f32 / 2f32.powi(6),
            par_p7: v.nvm_par_p7 as f32 / 2f32.powi(8),
            par_p8: v.nvm_par_p8 as f32 / 2f32.powi(15),
            par_p9: v.nvm_par_p9 as f32 / 2f32.powi(48),
            par_p10: v.nvm_par_p10 as f32 / 2f32.powi(48),
            par_p11: v.nvm_par_p11 as f32 / 2f32.powi(65),

            t_lin: 0f32,
        }
    }
}
