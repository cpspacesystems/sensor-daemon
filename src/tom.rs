use crate::{
    drivers::{
        bmp390::{Bmp390, Bmp390Frame},
        icm_20948::{Icm20948, Icm20948Frame},
    },
    i2c,
    multibus::MultiBus,
};
use std::{
    io,
    sync::{Arc, RwLock},
};

pub struct TomSensors {
    busses: [Arc<RwLock<i2c::Bus>>; 2],

    gyros: [Icm20948; 3],
    altimeters: [Bmp390; 3],

    gyro_data: Icm20948Frame,
    altimeter_data: Bmp390Frame,
}

impl TomSensors {
    /// Initialize TOM's sensors.
    pub fn init() -> io::Result<TomSensors> {
        let bus_1 = Arc::new(RwLock::new(i2c::Bus::open("/dev/i2c-1")?));
        let bus_2 = Arc::new(RwLock::new(i2c::Bus::open("/dev/i2c-2")?));

        let mut gyros = [
            Icm20948::open(bus_1.clone(), false)?,
            Icm20948::open(bus_1.clone(), true)?,
            Icm20948::open(bus_2.clone(), false)?,
        ];

        let mut altimeters = [
            Bmp390::open(bus_1.clone(), false)?,
            Bmp390::open(bus_2.clone(), false)?,
            Bmp390::open(bus_2.clone(), true)?,
        ];

        let gyro_data = median_gyro_data([gyros[0].read()?, gyros[1].read()?, gyros[2].read()?]);
        let altimeter_data = median_altimeter_data([
            altimeters[0].read()?,
            altimeters[1].read()?,
            altimeters[2].read()?,
        ]);

        Ok(TomSensors {
            busses: [bus_1, bus_2],
            gyros,
            altimeters,
            gyro_data,
            altimeter_data,
        })
    }

    /// Update all sensor data, median filtering each data point internally.
    pub fn update(&mut self) -> io::Result<()> {
        self.gyro_data = median_gyro_data([
            self.gyros[0].read()?,
            self.gyros[1].read()?,
            self.gyros[2].read()?,
        ]);

        self.altimeter_data = median_altimeter_data([
            self.altimeters[0].read()?,
            self.altimeters[1].read()?,
            self.altimeters[2].read()?,
        ]);

        Ok(())
    }

    /// Get the lastest gyro data. This data has been median filtered.
    pub fn gyro_data(&self) -> Icm20948Frame {
        self.gyro_data
    }

    /// Get the lastest altimeter data. This data has been median filtered.
    pub fn altimeter_data(&self) -> Bmp390Frame {
        self.altimeter_data
    }
}

fn median_altimeter_data(frames: [Bmp390Frame; 3]) -> Bmp390Frame {
    Bmp390Frame {
        tempurature: median_filter(frames.map(|f| f.tempurature)),
        pressure: median_filter(frames.map(|f| f.pressure)),
        altitude: median_filter(frames.map(|f| f.altitude)),
    }
}

fn median_gyro_data(frames: [Icm20948Frame; 3]) -> Icm20948Frame {
    Icm20948Frame {
        gyro_x: median_filter(frames.map(|f| f.gyro_x)),
        gyro_y: median_filter(frames.map(|f| f.gyro_y)),
        gyro_z: median_filter(frames.map(|f| f.gyro_z)),
        accel_x: median_filter(frames.map(|f| f.accel_x)),
        accel_y: median_filter(frames.map(|f| f.accel_y)),
        accel_z: median_filter(frames.map(|f| f.accel_z)),
    }
}

fn median_filter<T>(mut vals: [T; 3]) -> T
where
    T: PartialOrd,
{
    vals.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let [_min, median, _max] = vals;
    median
}
