#include <errno.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include "publish.h"
#include "config.h"
#include "sensor_frame.h"
#include "log.h"
#include "zenoh-pico/api/macros.h"
#include "zenoh-pico/api/primitives.h"
#include "zenoh-pico/api/types.h"
#include "zenoh-pico/config.h"
#include "zenoh-pico/utils/result.h"


/*
 * Shared memory backend.
 */

#ifdef PUBLISH_SHARED_MEM
#include <fcntl.h>
#include <sys/mman.h>
#include <pthread.h>
#endif  /* PUBLISH_SHARED_MEM */


/*
 * Zenoh backend.
 */

#ifdef PUBLISH_ZENOH

#include <zenoh-pico.h>

#endif  /* PUBLISH_ZENOH */


#define PUBLISH_ARG_NULL_CHECK(ptr) if (!ptr) return PUBLISH_INVALID_ARGUMENT;


/*
 * Shared memory backend implementation.
 */
 
#ifdef PUBLISH_SHARED_MEM

publish_error_t publisher_init(publisher_t* pub) {
	PUBLISH_ARG_NULL_CHECK(pub);
	
	pub->mem_fd = shm_open(
		PUBLISH_SHARED_MEM_NAME,
		O_CREAT | O_RDWR | O_EXCL | O_TRUNC
	);

	if (pub->mem_fd < 0) {
		goto handle_errno;
	}

	if (ftruncate(pub->mem_fd, PUBLISH_SHARED_MEM_SIZE)) {
		goto handle_errno;
	}

	pub->mutex = mmap(
		NULL,
		sizeof(pthread_mutex_t) + sizeof(sensor_frame_t),
		PROT_WRITE,
		MAP_SHARED,
		pub->mem_fd,
		0
	);

	pub->frame = (sensor_frame_t*)(pub->mutex + sizeof(pthread_mutex_t));

	if (pub->mutex == MAP_FAILED) {
		goto handle_errno;
	}

	int perror;
	if ((perror = pthread_mutex_init(pub->mutex, NULL)) != 0) {
		goto handle_perror;
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

handle_perror:
	switch (perror) {
		case ENOMEM:  return PUBLISH_NO_MEMORY;
		default:      return PUBLISH_UNKNWON;
	}
}

publish_error_t publisher_cleanup(publisher_t* pub) {
	PUBLISH_ARG_NULL_CHECK(pub);

	int perror;
	if ((perror = pthread_mutex_destroy(pub->mutex)) != 0) {
		goto handle_errno;
	}

	if (close(pub->mem_fd) < 0) {
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

handle_perror:
	switch (perror) {
		case EBUSY:  return PUBLISH_BUSY;
		default:     return PUBLISH_UNKNWON;
	}
}

publish_error_t publish_frame(publisher_t* pub, sensor_frame_t* frame) {
	PUBLISH_ARG_NULL_CHECK(pub);
	PUBLISH_ARG_NULL_CHECK(pub->mutex);
	PUBLISH_ARG_NULL_CHECK(pub->frame);

	int perror;
	if ((perror = pthread_mutex_lock(pub->mutex)) != 0) {
		LOG_ERROR("Failed to lock publish mutex!");
		goto handle_perror;
	}

	memcpy(pub->frame, frame, PUBLISH_SHARED_MEM_SIZE);

	if ((perror = pthread_mutex_unlock(pub->mutex)) != 0) {
		LOG_ERROR("Failed to unlock publish mutex!");
		goto handle_perror;
	}
	
	LOG_DEBUG("Published (`memcpy`) sensor frame (%u bytes).", sizeof(sensor_frame_t));
	return PUBLISH_OK;

handle_perror:
	switch (perror) {
		case EDEADLK: return PUBLISH_DEADLOCK;
		default:      return PUBLISH_UNKNWON;
	}
}

#endif  /* PUBLISH_SHARED_MEM */


/*
 * Zenoh backend implementation.
 */

#ifdef PUBLISH_ZENOH

publish_error_t publisher_init(publisher_t* pub) {
	z_owned_config_t config;
	z_result_t z_res;
	int session_open_retries = 0;


	/*
	 * Assuming that config succeeds, the only failures would be invalid arguments.
	 */

	z_config_default(&config);
	zp_config_insert(z_loan_mut(config), Z_CONFIG_MODE_KEY, Z_CONFIG_MODE_CLIENT);
	zp_config_insert(z_loan_mut(config), Z_CONFIG_SCOUTING_TIMEOUT_KEY, "16000");
	zp_config_insert(z_loan_mut(config), Z_CONFIG_CONNECT_KEY, "tcp/127.0.0.1:" ZENOH_TCP_LOCATOR_PORT);
	

	/*
	 * Open the Zenoh session, retrying up to a maximum number of times.
	 */
	 
retry_open_session:
	if ((z_res = z_open(&pub->session, z_move(config), NULL)) != Z_OK) {
		if (session_open_retries >= MAX_ZENOH_SESSION_OPEN_RETRIES) {
			LOG_ERROR("Failed to open Zenoh session (%i)! Maximum retries reached! Exiting ...", z_res);
			exit(z_res);
		}

		session_open_retries++;
		LOG_ERROR("Failed to open Zenoh session (%i)! Retrying in five seconds ...", z_res);
		sleep(5);

		goto retry_open_session;
	}

	LOG_INFO("Opened zenoh session!");


	/*
	 * Start some tasks for Zenoh. Pretty sure these are background threads that do
	 * most of Zenoh's work. These are needed to do just about anything with Zenoh.
	 */
	
	if ((z_res = zp_start_read_task(z_loan_mut(pub->session), NULL)) != Z_OK) {
		LOG_ERROR("Failed to start Zenoh's read task (%i)!", z_res);
		return PUBLISH_Z_TASK;
	}

	if ((z_res = zp_start_lease_task(z_loan_mut(pub->session), NULL)) != Z_OK) {
		LOG_ERROR("Failed to start Zenoh's lease task (%i)!", z_res);
		return PUBLISH_Z_TASK;
	}

	LOG_INFO("Started Zenoh tasks!");


	/*
	 * Declare the key we intend to publish (or 'put') new sensor frames over.
	 */

	z_view_keyexpr_t view_key;

	if ((z_res = z_view_keyexpr_from_str(&view_key, ZENOH_PUT_KEY)) != Z_OK) {
		return PUBLISH_Z_BAD_KEY;
	}

	z_res = z_declare_keyexpr(z_loan(pub->session), &pub->keyexpr, z_loan(view_key));

	if (z_res != Z_OK) {
		LOG_ERROR("Failed to declare Zenoh key (%i)!", z_res);
		return PUBLISH_Z_DECLARE;
	}
	
	return PUBLISH_OK;
}

publish_error_t publisher_cleanup(publisher_t* pub) {
	z_drop(z_move(pub->session));
	return PUBLISH_OK;
}

publish_error_t publish_frame(publisher_t* pub, sensor_frame_t* frame) {
	z_result_t z_res;
	z_owned_bytes_t bytes;
	z_bytes_copy_from_buf(&bytes, (void*)frame, sizeof(sensor_frame_t));
	
	if ((z_res = z_put(z_loan(pub->session), z_loan(pub->keyexpr), z_move(bytes), NULL)) != Z_OK) {
		LOG_ERROR("Failed to publish (`z_put`) (%i)!", z_res);
		return PUBLISH_UNKNWON;
	}
	
	LOG_DEBUG("Published (`z_put`) sensor frame (%u bytes).", sizeof(sensor_frame_t));
	return PUBLISH_OK;
}

#endif  /* PUBLISH_ZENOH */
