#include <errno.h>
#include <fcntl.h>
#include <string.h>
#include <sys/mman.h>
#include <unistd.h>

#include "sensord.h"
#include "publish.h"
#include "sensor_frame.h"

#define SENSORD_CHECK_ARG_NULL(arg) if (!arg) return PUBLISH_INVALID_ARGUMENT;

#ifdef PUBLISH_SHARED_MEM

sensord_error_t sensord_init(sensord_reciever_t* recv) {
	SENSORD_CHECK_ARG_NULL(recv);
	
	recv->mem_fd = shm_open(
		PUBLISH_SHARED_MEM_NAME,
		O_CREAT | O_RDONLY | O_EXCL | O_TRUNC
	);

	if (recv->mem_fd < 0) {
		goto handle_errno;
	}

	recv->mutex = mmap(
		NULL,
		sizeof(pthread_mutex_t) + sizeof(sensor_frame_t),
		PROT_WRITE,
		MAP_SHARED,
		recv->mem_fd,
		0
	);

	recv->frame = (sensor_frame_t*)(recv->mutex + sizeof(pthread_mutex_t));

	if (recv->mutex == MAP_FAILED) {
		goto handle_errno;
	}
	
	return PUBLISH_OK;

handle_errno:
	switch (errno) {
		case EACCES:  return PUBLISH_BAD_PERMISSIONS;
		case EEXIST:  return PUBLISH_ALREADY_OPEN;
		case ENFILE:
		case EMFILE:  return PUBLISH_TOO_MANY_FDS;
		case ENOSPC:  return PUBLISH_NO_STORAGE;
		case EINTR:   return PUBLISH_INTERRUPTED;
		case ENOMEM:  return PUBLISH_NO_MEMORY;
		case EDEADLK: return PUBLISH_DEADLOCK;
		default:      return PUBLISH_UNKNWON;
	}
}

sensord_error_t sensord_get_sensor_frame(sensord_reciever_t* recv, sensor_frame_t* frame) {
	SENSORD_CHECK_ARG_NULL(frame);
	SENSORD_CHECK_ARG_NULL(recv);
	SENSORD_CHECK_ARG_NULL(recv->mutex);
	SENSORD_CHECK_ARG_NULL(recv->frame);

	int perror;
	if ((perror = pthread_mutex_lock(recv->mutex)) != 0) {
		goto handle_perror;
	}

	memcpy(frame, recv->frame, sizeof(sensor_frame_t));

	if ((perror = pthread_mutex_unlock(recv->mutex)) != 0) {
		goto handle_perror;
	}
	
	return PUBLISH_OK;

handle_perror:
	switch (perror) {
		case EDEADLK: return PUBLISH_DEADLOCK;
		default:      return PUBLISH_UNKNWON;
	}
}

sensord_error_t sensord_cleanup(sensord_reciever_t* recv) {
	SENSORD_CHECK_ARG_NULL(recv);

	if (close(recv->mem_fd) < 0) {
		goto handle_errno;
	}

	return PUBLISH_OK;

handle_errno:
	switch (errno) {
		case EACCES: return PUBLISH_BAD_PERMISSIONS;
		case EEXIST: return PUBLISH_ALREADY_OPEN;
		case ENFILE:
		case EMFILE: return PUBLISH_TOO_MANY_FDS;
		case ENOSPC: return PUBLISH_NO_STORAGE;
		case EINTR:  return PUBLISH_INTERRUPTED;
		default:     return PUBLISH_UNKNWON;
	}
}

#endif  /* PUBLISH_SHARED_MEM */
