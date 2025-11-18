/*
 * Provides some convinience in packaging sensor data.
 */

#ifndef _SENSOR_FRAME_H
#define _SENSOR_FRAME_H

#include <sys/time.h>

#include "i2c.h"

/**
 * Frame of IMU data.
 */
typedef struct {
	double accel_x;  /* Acceleration in x direction in m/s^2. */
	double accel_y;  /* Acceleration in y direction in m/s^2. */
	double accel_z;  /* Acceleration in z direction in m/s^2. */
	double angle_x_rad;  /* Orientation about the x axis in radians. */
	double angle_y_rad;  /* Orientation about the y axis in radians. */
	double angle_z_rad;  /* Orientation about the z axis in radians. */
} sensor_imu_frame_t;

/**
 * Holistive frame of sensor data.
 */
typedef struct {
	sensor_imu_frame_t imu_frame;
	struct timeval measure_time;
} sensor_frame_t;

/**
 * Timestamp the given frame with the current time.
 */
void sensor_frame_timestamp(sensor_frame_t* frame);

/**
 * Assemble a `sensor_imu_frame_t` from the given ICM 20948 sensor.
 */
i2c_error_t sensor_imu_frame_record(i2c_bus_handle_t* bus, i2c_addr_t icm_20948, sensor_imu_frame_t* frame);

#endif  /* _SENSOR_FRAME_H */
