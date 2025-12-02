#include <errno.h>
#include <fcntl.h>
#include <stdatomic.h>
#include <stdio.h>
#include <string.h>
#include <sys/mman.h>
#include <unistd.h>

#include "sensord.h"
#include "config.h"
#include "publish.h"
#include "sensor_frame.h"

#define SENSORD_CHECK_ARG_NULL(arg) if (!arg) return PUBLISH_INVALID_ARGUMENT;


/*
 * Tempfile backend implementation.
 */

#ifdef PUBLISH_TMPFS

sensord_error_t sensord_init(sensord_reciever_t* recv) {
	SENSORD_CHECK_ARG_NULL(recv);

	recv->tempfile_fd = open(PUBLISH_TMPFILE_NAME, O_RDONLY);

	if (recv->tempfile_fd < 0) {
		goto handle_errno;
	}

	return PUBLISH_OK;
	
handle_errno:
	switch (errno) {
		case EACCES: return PUBLISH_BAD_PERMISSIONS;
		case EINTR:  return PUBLISH_INTERRUPTED;
		case EIO:    return PUBLISH_IO_ERROR;
		default:     return PUBLISH_UNKNWON;
	}
}

sensord_error_t sensord_get_sensor_frame(sensord_reciever_t* recv, sensor_frame_t* frame) {
	SENSORD_CHECK_ARG_NULL(recv);
	SENSORD_CHECK_ARG_NULL(frame);
	
	if (lseek(recv->tempfile_fd, 0, SEEK_SET) < 0) {
		goto handle_errno;
	}

	if (read(recv->tempfile_fd, (void*)frame, sizeof *frame) < 0) {
		goto handle_errno;
	}

	return PUBLISH_OK;

handle_errno:
	switch (errno) {
		case EACCES: return PUBLISH_BAD_PERMISSIONS;
		case EINTR:  return PUBLISH_INTERRUPTED;
		case EIO:    return PUBLISH_IO_ERROR;
		default:     return PUBLISH_UNKNWON;
	}
}

sensord_error_t sensord_cleanup(sensord_reciever_t* recv) {
	SENSORD_CHECK_ARG_NULL(recv);
	
	if (close(recv->tempfile_fd) < 0) {
		goto handle_errno;
	}

	return PUBLISH_OK;

handle_errno:
	switch (errno) {
		case EACCES: return PUBLISH_BAD_PERMISSIONS;
		case EINTR:  return PUBLISH_INTERRUPTED;
		case EIO:    return PUBLISH_IO_ERROR;
		default:     return PUBLISH_UNKNWON;
	}
}

#endif  /* PUBLISH_TMPFS */


/*
 * Shared memory backend implementation.
 */

#ifdef PUBLISH_SHM

sensord_error_t sensord_init(sensord_reciever_t* recv) {
	SENSORD_CHECK_ARG_NULL(recv);
	
	recv->mem_fd = shm_open(PUBLISH_SHM_NAME, O_RDONLY);

	if (recv->mem_fd < 0) {
		goto handle_errno;
	}

	recv->mutex = mmap(NULL, PUBLISH_SHM_SIZE, PROT_READ, MAP_SHARED, recv->mem_fd, 0);
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

#endif


/*
 * Zenoh backend implementation.
 */

#ifdef PUBLISH_ZENOH

sensord_error_t sensord_init(sensord_reciever_t* recv) {
	SENSORD_CHECK_ARG_NULL(recv);

	z_owned_config_t config;
	z_result_t z_res;
	int session_open_retries = 0;


	/*
	 * Assuming that config succeeds, the only failures would be invalid arguments.
	 */

	z_config_default(&config);
	zp_config_insert(z_loan_mut(config), Z_CONFIG_MODE_KEY, Z_CONFIG_MODE_PEER);
	// zp_config_insert(z_loan_mut(config), Z_CONFIG_SCOUTING_TIMEOUT_KEY, "16000");
	// zp_config_insert(z_loan_mut(config), Z_CONFIG_CONNECT_KEY, "tcp/127.0.0.1:7447");
	zp_config_insert(z_loan_mut(config), Z_CONFIG_MULTICAST_LOCATOR_KEY, Z_CONFIG_MULTICAST_LOCATOR_DEFAULT);
	zp_config_insert(z_loan_mut(config), Z_CONFIG_MULTICAST_SCOUTING_KEY, Z_CONFIG_MULTICAST_SCOUTING_DEFAULT);
	

	/*
	 * Open the Zenoh session, retrying up to a maximum number of times.
	 */
	 
retry_open_session:
	if ((z_res = z_open(&recv->session, z_move(config), NULL)) != Z_OK) {
		if (session_open_retries >= MAX_ZENOH_SESSION_OPEN_RETRIES) {
			exit(z_res);
		}

		session_open_retries++;
		sleep(1);

		goto retry_open_session;
	}


	/*
	 * Start some tasks for Zenoh. Pretty sure these are background threads that do
	 * most of Zenoh's work. These are needed to do just about anything with Zenoh.
	 */
	
	if ((z_res = zp_start_read_task(z_loan_mut(recv->session), NULL)) != Z_OK) {
		return PUBLISH_Z_TASK;
	}

	if ((z_res = zp_start_lease_task(z_loan_mut(recv->session), NULL)) != Z_OK) {
		return PUBLISH_Z_TASK;
	}


	/*
	 * Construct key expression in advance.
	 */

	if ((z_res = z_view_keyexpr_from_str(&recv->keyexpr, ZENOH_PUT_KEY)) != Z_OK) {
		return PUBLISH_Z_BAD_KEY;
	}

	return PUBLISH_OK;
}

static volatile atomic_bool sensord_get_callback_done = false;

static void sensord_get_callback(z_loaned_reply_t* reply, void* ctx) {
	sensor_frame_t* frame = (sensor_frame_t*)ctx;

	if (!z_reply_is_ok(reply)) {
		return;
	}

	const z_loaned_sample_t* sample = z_reply_ok(reply);
	const z_loaned_bytes_t* bytes = z_sample_payload(sample);

	z_bytes_reader_t reader = z_bytes_get_reader(bytes);
	z_bytes_reader_read(&reader, (uint8_t*)frame, sizeof *frame);
	sensord_get_callback_done = true;
}

sensord_error_t sensord_get_sensor_frame(sensord_reciever_t* recv, sensor_frame_t* frame) {
	SENSORD_CHECK_ARG_NULL(recv);
	SENSORD_CHECK_ARG_NULL(frame);

	z_result_t z_res;
	z_get_options_t opts;
	z_owned_closure_reply_t get_callback;

	z_get_options_default(&opts);
	z_closure(&get_callback, sensord_get_callback, NULL, (void*)frame);


	if ((z_res = z_get(z_loan(recv->session), z_loan(recv->keyexpr), "", z_move(get_callback), &opts)) != Z_OK) {
		return PUBLISH_Z_GET;
	}

	while (!sensord_get_callback_done) {}
	sensord_get_callback_done = false;

	return PUBLISH_OK;
}

sensord_error_t sensord_cleanup(sensord_reciever_t* recv) {
	SENSORD_CHECK_ARG_NULL(recv);
	z_drop(z_move(recv->session));
	return PUBLISH_OK;
}

#endif  /* PUBLISH_ZENOH */
