mod drivers;
mod frame;
mod i2c;
mod publish;

use crate::{frame::SensorFrameData, publish::Publisher};

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
        frame::timestamp_frame(frame)
            .and_then(|f| publisher.publish_frame(f))
            .unwrap();

        println!("Sent frame");

        frame.gyro_x *= 0.93;
        frame.gyro_y += 0.001;
        frame.gyro_z *= -0.993;
    }
}
