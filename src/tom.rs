use std::io;

use crate::{
    drivers::icm_20948,
    i2c,
    multibus::{CompoundAddress, MultiBus},
};

pub struct TomSensors {
    multibus: MultiBus<2>,
    gyros: [(CompoundAddress, Option<GyroFrame>); 3],
}

struct GyroFrame {}

impl TomSensors {
    pub fn init() -> io::Result<TomSensors> {
        let mut bus_1 = i2c::Bus::open("/dev/i2c-1")?;
        let mut bus_2 = i2c::Bus::open("/dev/i2c-2")?;

        let gyros = [
            (
                CompoundAddress(0, icm_20948::setup(&mut bus_1, false)?.into()),
                None,
            ),
            (
                CompoundAddress(0, icm_20948::setup(&mut bus_1, true)?.into()),
                None,
            ),
            (
                CompoundAddress(1, icm_20948::setup(&mut bus_2, false)?.into()),
                None,
            ),
        ];

        let multibus = MultiBus::new([bus_1, bus_2]);

        Ok(TomSensors { multibus, gyros })
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
