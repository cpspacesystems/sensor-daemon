#![feature(try_trait_v2)]

mod drivers;
mod fallible;
mod fb;
mod frame;
mod i2c;
mod publish;
mod tom;

use crate::{publish::Publisher, tom::TomSensors};

#[allow(dead_code, unused_imports)]
#[path = "./SensorFrame_generated.rs"]
mod sensor_frame_generated;

fn main() {
    let mut publisher = Publisher::new().expect("Should be able to create a publisher");
    let mut sensors = TomSensors::init().expect("Should be able to initialize sensors");

    loop {
        sensors.update();

        let frame = sensors.data();
        publisher.publish_frame(frame.clone()).unwrap();
    }
}
