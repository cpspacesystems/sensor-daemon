use std::io;

use crate::i2c;

pub struct MultiBus<const N: usize> {
    busses: [i2c::Bus; N],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompoundAddress(pub usize, pub i2c::Address);

impl<const N: usize> MultiBus<N> {
    pub fn new(busses: [i2c::Bus; N]) -> MultiBus<N> {
        MultiBus { busses }
    }

    pub fn get_bus_mut(&mut self, bus: usize) -> Option<&mut i2c::Bus> {
        self.busses.get_mut(bus)
    }

    pub fn set_device(&mut self, CompoundAddress(bus, addr): CompoundAddress) -> io::Result<()> {
        let bus_dev = self.get_bus_mut(bus).ok_or(io::Error::new(
            io::ErrorKind::AddrNotAvailable,
            "The `CompoundAddress` uses a bus which does not exist",
        ))?;

        bus_dev.set_device(addr)
    }

    pub fn read_register(
        &mut self,
        CompoundAddress(bus, addr): CompoundAddress,
        reg: u8,
        rx: &mut [u8],
    ) -> io::Result<()> {
        let bus_dev = self.get_bus_mut(bus).ok_or(io::Error::new(
            io::ErrorKind::AddrNotAvailable,
            "The `CompoundAddress` uses a bus which does not exist",
        ))?;

        bus_dev.read_register(addr, reg, rx)
    }

    pub fn write_read(
        &mut self,
        CompoundAddress(bus, addr): CompoundAddress,
        tx: &mut [u8],
        rx: &mut [u8],
    ) -> io::Result<()> {
        let bus_dev = self.get_bus_mut(bus).ok_or(io::Error::new(
            io::ErrorKind::AddrNotAvailable,
            "The `CompoundAddress` uses a bus which does not exist",
        ))?;

        bus_dev.write_read(addr, tx, rx)
    }

    pub fn write(
        &mut self,
        CompoundAddress(bus, addr): CompoundAddress,
        buf: &[u8],
    ) -> io::Result<()> {
        let bus_dev = self.get_bus_mut(bus).ok_or(io::Error::new(
            io::ErrorKind::AddrNotAvailable,
            "The `CompoundAddress` uses a bus which does not exist",
        ))?;

        bus_dev.write(addr, buf)
    }

    pub fn read(
        &mut self,
        CompoundAddress(bus, addr): CompoundAddress,
        buf: &mut [u8],
    ) -> io::Result<usize> {
        let bus_dev = self.get_bus_mut(bus).ok_or(io::Error::new(
            io::ErrorKind::AddrNotAvailable,
            "The `CompoundAddress` uses a bus which does not exist",
        ))?;

        bus_dev.read(addr, buf)
    }
}

impl Default for CompoundAddress {
    fn default() -> Self {
        Self(usize::max_value(), i2c::Address::SevenBit(0))
    }
}
