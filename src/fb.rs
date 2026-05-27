use crate::{
    drivers::{bmp390::Bmp390Frame, icm_20948::Icm20948Frame, neo_m9::NeoM9Frame},
    frame::SensorFrame,
    sensor_frame_generated::cpss::{self},
};
use flatbuffers::{self, WIPOffset};

/// Create the raw data used by flatbuffer to represent the given [`SensorFrame`].
///
/// [`SensorFrame`]: SensorFrame
pub fn create_fb_frame(
    SensorFrame {
        gyro_data:
            Icm20948Frame {
                gyro_x,
                gyro_y,
                gyro_z,
                accel_x,
                accel_y,
                accel_z,
            },
        altimeter_data:
            Bmp390Frame {
                temperature,
                pressure,
                altitude,
            },
        gps_data:
            NeoM9Frame {
                speed,
                heading,
                latitude,
                longitude,
                estimated_speed_error,
                estimated_longitude_error,
                estimated_latitude_error,
                satellites,
                valid_satellites,
            },
    }: SensorFrame,
) -> Vec<u8> {
    let mut fb_builder = flatbuffers::FlatBufferBuilder::with_capacity(256);

    let sensor_data = cpss::tom::sensord::SensorFrameData::new(
        gyro_x,
        gyro_y,
        gyro_z,
        accel_x,
        accel_y,
        accel_z,
        temperature,
        pressure,
        altitude,
        speed,
        heading,
        latitude,
        longitude,
        estimated_speed_error,
        estimated_latitude_error,
        estimated_longitude_error,
        satellites,
        valid_satellites,
    );

    let args = cpss::tom::sensord::SensorFrameArgs {
        data: Some(&sensor_data),
    };

    let _ = cpss::tom::sensord::SensorFrame::create(&mut fb_builder, &args);
    fb_builder.finish_minimal(WIPOffset::<()>::new(0));
    let data = fb_builder.finished_data();
    data.to_vec()
}
