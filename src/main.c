#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

#include "config.h"
#include "drivers/icm_20948.h"
#include "i2c.h"
#include "log.h"
#include "publish.h"
#include "sensor_frame.h"

int main() {
	log_init("sensor-daemon.log");
	log_configure(LOG_CONF_ALL | LOG_CONF_COLOR);


	/*
	 * Setup I2C bus.
	 */

	/*
	i2c_error_t i2c_err = I2C_OK;
	i2c_bus_handle_t i2c_bus_handle;
	char i2c_bus_name[16];

	if ((i2c_err = i2c_bus_auto_find(i2c_bus_name)) != I2C_OK) {
		LOG_ERROR("Failed to find I2C bus! Exiting ...");
		exit(i2c_err);
	}

	if ((i2c_err = i2c_bus_open(&i2c_bus_handle, i2c_bus_name)) != I2C_OK) {
		LOG_ERROR("Failed to open I2C bus! Exiting ...");
		exit(i2c_err);
	}
	*/


	/*
	 * Setup IMU, retrying up to a maximum number of times.
	 */

	 /*
	int imu_setup_retries = 0;
	i2c_addr_t imu_addr;

retry_setup_imu:
	if ((i2c_err = icm_setup(&i2c_bus_handle, &imu_addr, false)) != I2C_OK) {
		if (imu_setup_retries >= MAX_I2C_DEVICE_ADD_RETRIES) {
			LOG_ERROR("Failed to setup IMU! Maximum retries reached! Exiting ...");
			exit(i2c_err);
		}
		
		imu_setup_retries++;
		LOG_ERROR("Failed to setup IMU! Retrying in five seconds ...");
		sleep(5);

		goto retry_setup_imu;
	}
	*/


	/*
	 * Initialize publisher, what this does depends on what publisher backend you
	 * select. See the header file for more info about backends.
	 */

	publish_error_t publish_err = PUBLISH_OK;
	publisher_t publisher;

	if ((publish_err = publisher_init(&publisher)) != PUBLISH_OK) {
		LOG_ERROR("Failed to initialize publisher (%i)! Exiting ...", publish_err);
		exit(publish_err);
	}

	sensor_frame_t frame = {
		.imu_frame = {
			.accel_x = 0.0,
			.accel_y = 0.0,
			.accel_z = 0.0,

			.angle_x_rad = 0.0,
			.angle_y_rad = 0.0,
			.angle_z_rad = 0.0,
		}
	};
	
	sensor_frame_timestamp(&frame);
	publish_frame(&publisher, &frame);

	for (;;) {
		frame.imu_frame.accel_x *= 0.01;
		frame.imu_frame.accel_y += 0.1;
		frame.imu_frame.accel_z -= 0.1;
		frame.imu_frame.angle_x_rad += 0.1;
		frame.imu_frame.angle_y_rad -= 0.1;
		frame.imu_frame.angle_z_rad += 0.05;

		sensor_frame_timestamp(&frame);
		publish_frame(&publisher, &frame);
		usleep(100);
	}
}
