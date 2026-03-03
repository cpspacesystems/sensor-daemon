use std::io;

use crate::{i2c, multibus::MultiBus};

pub struct TomSensors {
    multibus: MultiBus<2>,
}

impl TomSensors {
    pub fn init() -> io::Result<TomSensors> {
        let mut bus_1 = i2c::Bus::open("/dev/i2c-1")?;
        let mut bus_2 = i2c::Bus::open("/dev/i2c-2")?;

        let multibus = MultiBus::new([bus_1, bus_2]);

        Ok(TomSensors { multibus })
    }
}

fn median_filter<T>(a: T, b: T, c: T) -> T
where
    T: Ord,
{
    let mut vals = [a, b, c];
    vals.sort_unstable();
    let [_min, median, _max] = vals;
    median
}
