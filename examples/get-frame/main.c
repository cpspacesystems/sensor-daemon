#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <sensord.h>

#include "publish.h"
#include "sensor_frame.h"

int main(void) {
	sensord_reciever_t recv;
	sensor_frame_t frame;
	sensord_error_t s_err;

	if ((s_err = sensord_init(&recv)) != PUBLISH_OK) {
		printf("Failed to init (%i)!\n", s_err);
		exit(-1);
	}

	struct timeval time;
	gettimeofday(&time, NULL);

	double sum = 0.0;

	for (int i = 0; i < 32; i++) {
		if ((s_err = sensord_get_sensor_frame(&recv, &frame)) != PUBLISH_OK) {
			printf("Failed to get (%i)!\n", s_err);
			exit(-1);
		}

		struct timeval time;
		gettimeofday(&time, NULL);

		printf("Incoming: Seconds: %ld, Micros: %u\n", frame.measure_time.tv_sec, frame.measure_time.tv_usec);
		printf("Local:    Seconds: %ld, Micros: %u\n", time.tv_sec, time.tv_usec);
		
		printf("Staleness (s): %f\n", sensor_frame_staleness(&frame));
		sum += sensor_frame_staleness(&frame);
		usleep(20000);
	}

	printf("Average Staleness (s): %f\n", sum / 32);
}
