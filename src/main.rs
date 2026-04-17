mod drivers;
mod frame;
mod i2c;
mod publish;
mod tom;

use crate::{publish::Publisher, tom::TomSensors};

fn main() {
    let mut publisher = Publisher::new().expect("Should be able to create a publisher");
    let mut sensors = TomSensors::init().expect("Should be able to initialize sensors");

    loop {
        sensors
            .update()
            .expect("Should be able to update sensor data");

        println!("Updated sensor frame");

        let frame = sensors.data();
        let stamped_frame = frame::timestamp_frame(frame).unwrap();
        publisher.publish_frame(stamped_frame.clone()).unwrap();

        println!("Sent frame at {:?}", stamped_frame.time());
    }
}
