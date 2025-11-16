#ifndef _SENSORD_H
#define _SENSORD_H

#include "sensor_frame.h"
#include "publish.h"

typedef publish_error_t sensord_error_t;

typedef struct {
#ifdef PUBLISH_SHARED_MEM
	int mem_fd;
	pthread_mutex_t* mutex;
	sensor_frame_t* frame;
#endif  /* PUBLISH_SHARED_MEM */

} sensord_reciever_t;

/**
 * Initialize the required recourses for recieving data from sensor-daemon into
 * the pointer to an uninitialized `sensord_reciever_t`.
 */
sensord_error_t sensord_init(sensord_reciever_t* recv);

/**
 * Gets the latest `sensor_frame_t` from sensor-daemon and places it into the
 * given pointer.
 */
sensord_error_t sensord_get_sensor_frame(sensord_reciever_t* recv, sensor_frame_t* frame);

/**
 * Free up recourses used by the given `sensord_reciever_t`.
 */
sensord_error_t sensord_cleanup(sensord_reciever_t* recv);

#endif  /* _SENSORD_H */
