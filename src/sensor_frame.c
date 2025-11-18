#include "sensor_frame.h"
#include "drivers/icm_20948.h"
#include "i2c.h"

#define MBIND(e) { i2c_error_t err = e; if (e != I2C_OK) return e; }

void sensor_frame_timestamp(sensor_frame_t* frame) {
	gettimeofday(&frame->measure_time, NULL);
}

i2c_error_t sensor_imu_frame_record(i2c_bus_handle_t* bus, i2c_addr_t icm_20948, sensor_imu_frame_t* frame) {
	MBIND(icm_get_accel_x(bus, icm_20948, &frame->accel_x));
	MBIND(icm_get_accel_y(bus, icm_20948, &frame->accel_y));
	MBIND(icm_get_accel_z(bus, icm_20948, &frame->accel_z));

	MBIND(icm_get_gyro_x(bus, icm_20948, &frame->angle_x_rad));
	MBIND(icm_get_gyro_y(bus, icm_20948, &frame->angle_y_rad));
	MBIND(icm_get_gyro_z(bus, icm_20948, &frame->angle_z_rad));

	return I2C_OK;
}
