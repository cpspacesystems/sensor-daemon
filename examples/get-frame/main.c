#include "publish.h"
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <sensord.h>

int main(void) {
	sensord_reciever_t recv;
	sensor_frame_t frame;
	sensord_error_t s_err;

	if ((s_err = sensord_init(&recv)) != PUBLISH_OK) {
		printf("Failed to init (%i)!\n", s_err);
		exit(-1);
	}

	if ((s_err = sensord_get_sensor_frame(&recv, &frame)) != PUBLISH_OK) {
		printf("Failed to get (%i)!\n", s_err);
		exit(-1);
	}

	printf("Sensor Frame:\n");
	printf("\tGyro X: %f\n", frame.imu_frame.angle_x_rad);
	printf("\tGyro Y: %f\n", frame.imu_frame.angle_y_rad);
	printf("\tGyro Z: %f\n", frame.imu_frame.angle_z_rad);
	printf("\tAccel X: %f\n", frame.imu_frame.accel_x);
	printf("\tAccel Y: %f\n", frame.imu_frame.accel_y);
	printf("\tAccel Z: %f\n", frame.imu_frame.accel_z);
}
