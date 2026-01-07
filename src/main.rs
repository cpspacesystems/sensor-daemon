mod drivers;
mod i2c;
mod publish;

use crate::publish::Publisher;
use sensor_daemon::frame::{self, SensorFrameData};

fn main() {
    let mut publisher = Publisher::new().expect("Should be able to create a publisher");

    let mut frame = SensorFrameData {
        accel_x: 1.0,
        accel_y: 0.0,
        accel_z: -2.0,
        gyro_x: 1.0,
        gyro_y: 0.0,
        gyro_z: -2.0,
    };

    loop {
        let stamped_frame = frame::timestamp_frame(frame).unwrap();
        publisher.publish_frame(stamped_frame.clone()).unwrap();

        println!("Sent frame at {:?}", stamped_frame.time());

        frame.gyro_x *= 0.93;
        frame.gyro_y += 0.001;
        frame.gyro_z *= -0.993;
    }
}
