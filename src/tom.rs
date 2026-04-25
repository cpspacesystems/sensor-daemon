use crate::{
    drivers::{
        bmp390::{self, Bmp390, Bmp390Frame},
        icm_20948::{self, Icm20948, Icm20948Frame},
        neo_m9::{NeoM9, NeoM9Frame},
    },
    fallible::FallibleDevice,
    frame::SensorFrameData,
    i2c,
};
use std::{
    convert::Infallible,
    io,
    sync::{Arc, RwLock},
};

/// Type for all the TOM specific stuff, this is where sensor data is updated, stored, filtered, and
/// managed. Most of the time [`TomSensors`]'s functions will swallow errors, rather than propogate
/// them, since the whole point is redundancy here.
///
/// [`TomSensors`]: TomSensors
pub struct TomSensors {
    _busses: [Arc<RwLock<i2c::Bus>>; 2],

    gyros: [FallibleDevice<Icm20948, io::Result<Infallible>>; 3],
    altimeters: [FallibleDevice<Bmp390, io::Result<Infallible>>; 3],
    gps: FallibleDevice<NeoM9, Result<Infallible, gpsd_client::GPSError>>,

    gyro_data: Icm20948Frame,
    altimeter_data: Bmp390Frame,
    gps_data: NeoM9Frame,
}

impl TomSensors {
    /// Initialize TOM's sensors.
    pub fn init() -> io::Result<TomSensors> {
        let bus_1 = Arc::new(RwLock::new(i2c::Bus::open("/dev/i2c-2")?));
        let bus_2 = Arc::new(RwLock::new(i2c::Bus::open("/dev/i2c-3")?));

        let mut gyros = [
            FallibleDevice::new(
                Icm20948::open(bus_1.clone(), false)
                    .and_then(|icm| {
                        icm.with_accelerometer_scale(icm_20948::AccelerometerScale::Scale16G)
                    })
                    .and_then(|icm| {
                        icm.with_gyro_scale(icm_20948::GyroScale::Scale2000DegreesPerSecond)
                    }),
                "ICM20948 @ I2C 2 0x68",
            ),
            FallibleDevice::new(
                Icm20948::open(bus_1.clone(), true)
                    .and_then(|icm| {
                        icm.with_accelerometer_scale(icm_20948::AccelerometerScale::Scale16G)
                    })
                    .and_then(|icm| {
                        icm.with_gyro_scale(icm_20948::GyroScale::Scale2000DegreesPerSecond)
                    }),
                "ICM20948 @ I2C 2 0x69",
            ),
            FallibleDevice::new(
                Icm20948::open(bus_2.clone(), false)
                    .and_then(|icm| {
                        icm.with_accelerometer_scale(icm_20948::AccelerometerScale::Scale16G)
                    })
                    .and_then(|icm| {
                        icm.with_gyro_scale(icm_20948::GyroScale::Scale2000DegreesPerSecond)
                    }),
                "ICM20948 @ I2C 3 0x68",
            ),
        ];

        let mut altimeters = [
            FallibleDevice::new(
                Bmp390::open(bus_1.clone(), true).and_then(|bmp| {
                    bmp.with_oversampling(
                        bmp390::Oversampling::Oversample4x,
                        bmp390::Oversampling::NoOversampling,
                    )
                }),
                "BMP390 @ I2C 2 0x77",
            ),
            FallibleDevice::new(
                Bmp390::open(bus_2.clone(), false).and_then(|bmp| {
                    bmp.with_oversampling(
                        bmp390::Oversampling::Oversample4x,
                        bmp390::Oversampling::NoOversampling,
                    )
                }),
                "BMP390 @ I2C 3 0x76",
            ),
            FallibleDevice::new(
                Bmp390::open(bus_2.clone(), true).and_then(|bmp| {
                    bmp.with_oversampling(
                        bmp390::Oversampling::Oversample4x,
                        bmp390::Oversampling::NoOversampling,
                    )
                }),
                "BMP390 @ I2C 3 0x77",
            ),
        ];

        let mut gps = FallibleDevice::new(NeoM9::open(), "NEO M9");

        let gyro_data = median_gyro_data([
            gyros[0].run(Icm20948::read).unwrap_or_default(),
            gyros[1].run(Icm20948::read).unwrap_or_default(),
            gyros[2].run(Icm20948::read).unwrap_or_default(),
        ]);

        let altimeter_data = median_altimeter_data([
            altimeters[0].run(Bmp390::read).unwrap_or_default(),
            altimeters[1].run(Bmp390::read).unwrap_or_default(),
            altimeters[2].run(Bmp390::read).unwrap_or_default(),
        ]);

        let gps_data = gps.run(NeoM9::read).unwrap_or_default();

        Ok(TomSensors {
            _busses: [bus_1, bus_2],
            gyros,
            altimeters,
            gps,
            gyro_data,
            altimeter_data,
            gps_data,
        })
    }

    /// Get the most recently updated data available. Call [`TomSensors::update`] to retrieve new
    /// data which can then be gotten with this method.
    ///
    /// [`TomSensors::update`]: TomSensors::update
    pub fn data(&self) -> SensorFrameData {
        SensorFrameData {
            gyro_data: self.gyro_data,
            altimeter_data: self.altimeter_data,
            gps_data: self.gps_data,
        }
    }

    /// Update all sensor data, median filtering each data point internally.
    pub fn update(&mut self) {
        self.gyro_data = median_gyro_data([
            self.gyros[0].run(Icm20948::read).unwrap_or_default(),
            self.gyros[1].run(Icm20948::read).unwrap_or_default(),
            self.gyros[2].run(Icm20948::read).unwrap_or_default(),
        ]);

        self.altimeter_data = median_altimeter_data([
            self.altimeters[0].run(Bmp390::read).unwrap_or_default(),
            self.altimeters[1].run(Bmp390::read).unwrap_or_default(),
            self.altimeters[2].run(Bmp390::read).unwrap_or_default(),
        ]);

        self.gps_data = self.gps.run(NeoM9::read).unwrap_or_default();
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
