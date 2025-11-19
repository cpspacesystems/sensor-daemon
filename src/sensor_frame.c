#include "sensor_frame.h"
#include "drivers/icm_20948.h"
#include "i2c.h"

#define MBIND(e) { i2c_error_t err = e; if (e != I2C_OK) return e; }

static double timeval_seconds(struct timeval time) {
	return (double)time.tv_sec + ((double)time.tv_usec * 0.000001);
}

void sensor_frame_timestamp(sensor_frame_t* frame) {
	gettimeofday(&frame->measure_time, NULL);
}

double sensor_frame_staleness(sensor_frame_t* frame) {
	struct timeval current_time;
	gettimeofday(&current_time, NULL);
	return timeval_seconds(current_time) - timeval_seconds(frame->measure_time);
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
