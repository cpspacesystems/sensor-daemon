use crate::{
    drivers::{bmp390::Bmp390Frame, icm_20948::Icm20948Frame, neo_m9::NeoM9Frame},
    frame::SensorFrame,
    sensor_frame_generated::cpss::{self},
};
use flatbuffers;

/// Create the raw data used by flatbuffer to represent the given [`SensorFrame`].
///
/// [`SensorFrame`]: SensorFrame
pub fn create_fb_frame(frame: SensorFrame) -> Vec<u8> {
    let mut fb_builder = flatbuffers::FlatBufferBuilder::with_capacity(256);
    let _ = cpss::tom::sensord::SensorFrame::create(&mut fb_builder, &frame.into());
    let data = fb_builder.finished_data();
    data.to_vec()
}

impl From<SensorFrame> for cpss::tom::sensord::SensorFrameArgs {
    fn from(value: SensorFrame) -> Self {
        cpss::tom::sensord::SensorFrameArgs {
            gyro_x: value.gyro_data.gyro_x,
            gyro_y: value.gyro_data.gyro_y,
            gyro_z: value.gyro_data.gyro_z,
            accel_x: value.gyro_data.accel_x,
            accel_y: value.gyro_data.accel_y,
            accel_z: value.gyro_data.accel_z,

            temperature: value.altimeter_data.temperature,
            pressure: value.altimeter_data.pressure,
            altitude: value.altimeter_data.altitude,

            speed: value.gps_data.speed,
            heading: value.gps_data.heading,
            latitude: value.gps_data.latitude,
            longitude: value.gps_data.longitude,
            estimated_speed_error: value.gps_data.estimated_speed_error,
            estimated_latitude_error: value.gps_data.estimated_latitude_error,
            estimated_longitude_error: value.gps_data.estimated_longitude_error,
            satellites: value.gps_data.satellites,
            valid_satellites: value.gps_data.valid_satellites,
        }
    }
}
