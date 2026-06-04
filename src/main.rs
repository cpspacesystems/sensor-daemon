#![feature(try_trait_v2)]

mod drivers;
mod fallible;
mod fb;
mod frame;
mod i2c;
mod publish;
mod tom;

use std::env;

use crate::{publish::Publisher, tom::TomSensors};

#[allow(dead_code, unused_imports)]
#[path = "./SensorFrame_generated.rs"]
mod sensor_frame_generated;

fn main() {
    if env::args().any(|arg| arg == "--spoof") {
        let mut publisher = Publisher::new().expect("Should be able to create a publisher");
        let mut has_allocated_tism = publisher.has_allocated_tism();

        let mut frame = frame::SensorFrame {
            gyro_data: drivers::icm_20948::Icm20948Frame {
                gyro_x: 1.0,
                gyro_y: 1.0,
                gyro_z: 1.0,
                accel_x: 1.0,
                accel_y: 1.0,
                accel_z: 1.0,
            },
            altimeter_data: drivers::bmp390::Bmp390Frame {
                temperature: 1.0,
                pressure: 1.0,
                altitude: 1.0,
            },
            gps_data: drivers::neo_m9::NeoM9Frame {
                speed: 1.0,
                heading: 1.0,
                latitude: 1.0,
                longitude: 1.0,
                estimated_speed_error: 1.0,
                estimated_longitude_error: 1.0,
                estimated_latitude_error: 1.0,
                satellites: 0,
                valid_satellites: 0,
            },
        };

        loop {
            frame.gyro_data.gyro_x += 0.001;
            frame.gyro_data.gyro_y *= 1.0001;
            frame.gyro_data.gyro_z *= 0.999;
            frame.gyro_data.accel_x += 0.001;
            frame.gyro_data.accel_y *= 1.0001;
            frame.gyro_data.accel_z *= 0.999;

            publisher.publish_frame(frame.clone()).unwrap();

            let allocated_status = publisher.has_allocated_tism();

            if has_allocated_tism != allocated_status && allocated_status {
                println!("Allocated the TISM allocation!");
                has_allocated_tism = allocated_status;
            }
        }
    }

    let mut publisher = Publisher::new().expect("Should be able to create a publisher");
    let mut sensors = TomSensors::init().expect("Should be able to initialize sensors");

    let mut has_allocated_tism = publisher.has_allocated_tism();

    loop {
        sensors.update();

        let frame = sensors.data();
        publisher.publish_frame(frame.clone()).unwrap();

        let allocated_status = publisher.has_allocated_tism();

        if has_allocated_tism != allocated_status && allocated_status {
            println!("Allocated the TISM allocation!");
            has_allocated_tism = allocated_status;
        }
    }
}
