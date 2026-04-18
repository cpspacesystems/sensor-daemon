#![cfg(target_family = "unix")]

use libc;
use std::{
    fs::File,
    io::{self, Read},
    os::{fd::AsRawFd, unix::fs::OpenOptionsExt},
    path::Path,
};

/// An I2C bus.
#[derive(Debug)]
pub struct Bus {
    file: File,
    funcs: usize,
}

/// The address of a device on the [`i2c::Bus`].
///
/// [`i2c::Bus`]: Bus
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Address {
    SevenBit(u8),
    TenBit(u16),
}

/// Possible address widths over I2C.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AddressWidth {
    SevenBit = 0,
    TenBit = 1,
}

impl Bus {
    /// Open an I2C bus at the given index.
    pub fn open(path: impl AsRef<Path>) -> io::Result<Bus> {
        let file = File::options()
            .write(true)
            .read(true)
            .custom_flags(libc::O_NONBLOCK)
            .open(path)?;
        let mut funcs: usize = 0;

        unsafe {
            if libc::ioctl(file.as_raw_fd(), I2C_FUNCS, [&raw mut funcs]) == -1 {
                return Err(io::Error::last_os_error());
            }
        }

        if !functionality_present(funcs, Functionality::I2c) {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "Given I2C bus does not support I2C, ¯\\_(ツ)_/¯",
            ));
        }

        Ok(Bus { file, funcs })
    }

    /// Add a slave device to the [`i2c::Bus`].
    ///
    /// [`i2c::Bus`]: Bus
    pub fn add_device(&mut self, addr: Address) -> io::Result<()> {
        let address = match addr {
            Address::SevenBit(a) => a as _,
            Address::TenBit(a) => a,
        };

        unsafe {
            if libc::ioctl(self.file.as_raw_fd(), I2C_SLAVE, [address]) == -1 {
                return Err(io::Error::last_os_error());
            }
        }

        Ok(())
    }

    /// Set the current device for the [`i2c::Bus`] via its [`Address`].
    ///
    /// [`i2c::Bus`]: Bus
    /// [`Address`]: Address
    pub fn set_device(&mut self, addr: Address) -> io::Result<()> {
        match addr {
            Address::SevenBit(address) => {
                self.set_address_width(AddressWidth::SevenBit)?;
                self.set_device_unchecked(address as _)
            }

            Address::TenBit(address) => {
                self.set_address_width(AddressWidth::TenBit)?;
                self.set_device_unchecked(address)
            }
        }
    }

    /// Set a device at the given `address` to be current, skipping the usual checks for
    /// functionality.
    fn set_device_unchecked(&mut self, address: u16) -> io::Result<()> {
        unsafe {
            match libc::ioctl(self.file.as_raw_fd(), I2C_SLAVE, [address]) {
                -1 => Err(io::Error::last_os_error()),
                _ => Ok(()),
            }
        }
    }

    /// Set the current address width of the given [`i2c::Bus`].
    ///
    /// [`i2c::Bus`]: Bus
    fn set_address_width(&mut self, width: AddressWidth) -> io::Result<()> {
        if width == AddressWidth::TenBit
            && !functionality_present(self.funcs, Functionality::TenBitAddress)
        {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "Given I2C bus does not support ten bit addressing",
            ));
        }

        unsafe {
            match libc::ioctl(self.file.as_raw_fd(), I2C_TENBIT, [width]) {
                -1 => Err(io::Error::last_os_error()),
                _ => Ok(()),
            }
        }
    }

    /// Read out the value of the given register `reg` from the device at the given [`Address`].
    /// This function is a wrapper over [`i2c::Bus::write_read`] for use with the common pattern of
    /// register access on I2C devices.
    ///
    /// [`Address`]: Address
    /// [`i2c::Bus::write_read`]: Bus::write_read
    pub fn read_register(&mut self, addr: Address, reg: u8, rx: &mut [u8]) -> io::Result<()> {
        self.write_read(addr, &mut [reg], rx)
    }

    /// Perform a write over I2C to the given [`Address`], and then read a response off from the
    /// same [`Address`].
    ///
    /// [`Address`]: Address
    pub fn write_read(&mut self, addr: Address, tx: &mut [u8], rx: &mut [u8]) -> io::Result<()> {
        let address = match addr {
            Address::SevenBit(a) => a as _,
            Address::TenBit(a) => a,
        };

        const READ: u16 = 0x0001;
        const TEN_BIT: u16 = 0x0010;

        let flags = match addr {
            Address::SevenBit(_) => 0,
            Address::TenBit(_) => TEN_BIT,
        };

        let tx_msg = I2cMessage {
            address,
            flags,
            len: tx.len() as _,
            buf: tx.as_mut_ptr(),
        };

        let rx_msg = I2cMessage {
            address,
            flags: flags | READ,
            len: rx.len() as _,
            buf: rx.as_mut_ptr(),
        };

        let mut messages = [tx_msg, rx_msg];
        let mut data = I2cReadWriteData {
            messages: messages.as_mut_ptr(),
            num_messages: messages.len() as _,
        };

        unsafe {
            match libc::ioctl(self.file.as_raw_fd(), I2C_RDWR, &raw mut data) {
                -1 => Err(io::Error::last_os_error()),
                _ => Ok(()),
            }
        }
    }

    /// Write the given buffer to the device at the given [`Address`].
    ///
    /// [`Address`]: Address
    pub fn write(&mut self, addr: Address, buf: &mut [u8]) -> io::Result<()> {
        let mut _buf = [0u8; 16];
        self.write_read(addr, buf, &mut _buf)
    }
}

/// Returns `true` if the given [`Functionality`] is present in the given `usize`, which will be
/// taken to be a collection of bit flags with values corrosponding to variants of
/// [`Functionality`].
///
/// [`Functionality`]: Functionality
fn functionality_present(funcs: usize, f: Functionality) -> bool {
    funcs & f as usize == f as usize
}

/// Data required by `ioctl` for making combined read and write actions over I2C. This is the
/// `i2c_rdwr_ioctl_data` struct from the Linux I2C implementation.
#[repr(C)]
struct I2cReadWriteData {
    messages: *mut I2cMessage,
    num_messages: u32,
}

/// `i2c_msg` struct from the Linux I2C implementation.
#[repr(C)]
struct I2cMessage {
    address: u16,
    flags: u16,
    len: u16,
    buf: *mut u8,
}

const I2C_RETRIES: libc::c_ulong = 0x0701;
const I2C_TIMEOUT: libc::c_ulong = 0x0702;
const I2C_SLAVE: libc::c_ulong = 0x0703;
const I2C_SLAVE_FORCE: libc::c_ulong = 0x0706;
const I2C_TENBIT: libc::c_ulong = 0x0704;
const I2C_FUNCS: libc::c_ulong = 0x0705;
const I2C_RDWR: libc::c_ulong = 0x0707;
const I2C_PEC: libc::c_ulong = 0x0708;
const I2C_SMBUS: libc::c_ulong = 0x0720;

/// A subset of the functionality that an I2C bus (I2C master device) might have,
#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Functionality {
    /// Supports I2C, one would hope this is the case for an I2C bus...
    I2c = 0x00000001,

    /// Supports ten bit device addresses.
    TenBitAddress = 0x00000002,

    ProtocolMangling = 0x00000004,

    NoStart = 0x00000010,

    Slave = 0x00000020,
}
