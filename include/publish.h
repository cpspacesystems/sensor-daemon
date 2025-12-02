/*
 * Data publishing for consuming processes. Definitions determine the backend
 * for this interface.
 *
 * Defining `PUBLISH_SHARED_MEM` will enable the shared memory backend. Defining
 * `PUBLISH_ZENOH` will enable the Zenoh backend, via zenoh-pico.
 */
 
#ifndef _PUBLISH_H
#define _PUBLISH_H

#include <stdio.h>
#include <stddef.h>
#include "sensor_frame.h"

// #define PUBLISH_ZENOH
#define PUBLISH_TMPFS


/*
 * Zenoh backend definitions.
 */
 
#ifdef PUBLISH_ZENOH

#if defined(__apple__)
#define ZENOH_MACOS
#elif defined(__unix__)
#define ZENOH_BSD
#else
#define ZENOH_LINUX
#endif

#include <zenoh-pico.h>

#endif  /* PUBLISH_ZENOH */


/*
 * Tempfile backend definitions.
 */

#ifdef PUBLISH_TMPFS

#define PUBLISH_TMPFILE_NAME "/tmp/sensord.temp"

#endif  /* PUBLISH_TMPFS */


/**
 * Data needed to publishing. The fields of this type depend on the selected
 * backend.
 */
typedef struct {
#ifdef PUBLISH_ZENOH
	z_owned_session_t session;
	z_owned_keyexpr_t keyexpr;
#endif  /* PUBLISH_ZENOH */

#ifdef PUBLISH_TMPFS
	int tempfile_fd;
#endif  /* PUBLISH_TMPFS */
} publisher_t;

typedef enum {
	PUBLISH_OK = 0,

	/**
	 * Lacking necessary permissions to open a recourse.
	 */
	PUBLISH_BAD_PERMISSIONS = -1,

	/**
	 * Open attempt for shared memory failed due to it already being open.
	 */
	PUBLISH_ALREADY_OPEN = -2,

	/**
	 * To many files are open, either for this process or the system as a whole.
	 */
	PUBLISH_TOO_MANY_FDS = -3,

	/**
	 * Not enough storage space for operation.
	 */
	PUBLISH_NO_STORAGE = -4,

	/**
	 * Signal interrupted operation.
	 */
	PUBLISH_INTERRUPTED = -5,

	/**
	 * Invalid argument to function.
	 */
	PUBLISH_INVALID_ARGUMENT = -6,

	/**
	 * I/O error occured during operation.
	 */
	PUBLISH_IO_ERROR = -7,

	/**
	 * Not enough memory for operation.
	 */
	PUBLISH_NO_MEMORY = -8,


#ifdef PUBLISH_TMPFS

#endif  /* PUBLISH_TMPFS */


#ifdef PUBLISH_ZENOH
	/**
	 * Zenoh get failed.
	 */
	PUBLISH_Z_GET = -124,

	/**
	 * Failure to declare Zenoh key.
	 */
	PUBLISH_Z_DECLARE = -125,

	/**
	 * Bad Zenoh key.
	 */
	PUBLISH_Z_BAD_KEY = -126,

	/**
	 * Failed to start a Zenoh task/
	 */
	PUBLISH_Z_TASK = -127,
#endif  /* PUBLISH_ZENOH */
	

	PUBLISH_UNKNWON = -128,
} publish_error_t;

/**
 * Initialize a `publisher_t` in the pointer given.
 */
publish_error_t publisher_init(publisher_t* publisher);

/**
 * Free the recourses used by the given publisher.
 */
publish_error_t publisher_cleanup(publisher_t* publisher);

/**
 * Publish the given frame.
 */
publish_error_t publish_frame(publisher_t* pub, sensor_frame_t* frame);

#endif  /* _PUBLISH_H */
