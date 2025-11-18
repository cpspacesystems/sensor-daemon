#include <errno.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <fcntl.h>
#include <sys/stat.h>

#include "publish.h"
#include "sensor_frame.h"
#include "log.h"

#ifdef PUBLISH_ZENOH
#include "config.h"
#include <zenoh-pico.h>
#endif  /* PUBLISH_ZENOH */

#define PUBLISH_ARG_NULL_CHECK(ptr) if (!ptr) return PUBLISH_INVALID_ARGUMENT;


/*
 * Tempfile backend implementation.
 */

#ifdef PUBLISH_TMPFS

publish_error_t publisher_init(publisher_t* publisher) {
	PUBLISH_ARG_NULL_CHECK(publisher);
	
	// strcpy(publisher->tempfile_name, PUBLISH_TMPFILE_NAME);
	// publisher->tempfile_fd = mkstemp(publisher->tempfile_name);

	publisher->tempfile_fd = open(PUBLISH_TMPFILE_NAME, O_RDWR | O_CREAT);

	if (publisher->tempfile_fd < 0) {
		LOG_ERROR("Could not open tempfile (%s) for publishing (%i)!", PUBLISH_TMPFILE_NAME, errno);
		goto handle_errno;
	}

	LOG_INFO("Opened '%s' for publishing.", PUBLISH_TMPFILE_NAME);

	int permissions = S_IRWXU | S_IRWXG | S_IROTH;
	
	if (fchmod(publisher->tempfile_fd, permissions) < 0) {
		LOG_ERROR("Failed to set permissions for '%s' (%i)!", PUBLISH_TMPFILE_NAME, errno);
		goto handle_errno;
	}

	LOG_INFO("Set permissions for tempfile '%s' to %i", PUBLISH_TMPFILE_NAME, permissions);

	return PUBLISH_OK;
	
handle_errno:
	switch (errno) {
		case EACCES: return PUBLISH_BAD_PERMISSIONS;
		case EINTR:  return PUBLISH_INTERRUPTED;
		case EIO:    return PUBLISH_IO_ERROR;
		default:     return PUBLISH_UNKNWON;
	}
}

publish_error_t publisher_cleanup(publisher_t* publisher) {
	PUBLISH_ARG_NULL_CHECK(publisher);

	if (close(publisher->tempfile_fd) < 0) {
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

publish_error_t publish_frame(publisher_t* publisher, sensor_frame_t* frame) {
	PUBLISH_ARG_NULL_CHECK(publisher);

	if (lseek(publisher->tempfile_fd, 0, SEEK_SET) < 0) {
		goto handle_errno;
	}

	ssize_t n;

	if ((n = write(publisher->tempfile_fd, (void*)frame, sizeof *frame)) < 0) {
		goto handle_errno;
	}

	LOG_DEBUG("Published (`write`) sensor frame (%i bytes).", n);

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
 * Zenoh backend implementaton.
 */

#ifdef PUBLISH_ZENOH

publish_error_t publisher_init(publisher_t* pub) {
	PUBLISH_ARG_NULL_CHECK(pub);

	z_owned_config_t config;
	z_result_t z_res;
	int session_open_retries = 0;


	/*
	 * Assuming that config succeeds, the only failures would be invalid arguments.
	 */

	z_config_default(&config);
	zp_config_insert(z_loan_mut(config), Z_CONFIG_MODE_KEY, Z_CONFIG_MODE_CLIENT);
	zp_config_insert(z_loan_mut(config), Z_CONFIG_SCOUTING_TIMEOUT_KEY, "16000");
	zp_config_insert(z_loan_mut(config), Z_CONFIG_CONNECT_KEY, "tcp/10.144.239.91:7447");
	// zp_config_insert(z_loan_mut(config), Z_CONFIG_MULTICAST_LOCATOR_KEY, Z_CONFIG_MULTICAST_LOCATOR_DEFAULT);
	// zp_config_insert(z_loan_mut(config), Z_CONFIG_MULTICAST_SCOUTING_KEY, Z_CONFIG_MULTICAST_SCOUTING_DEFAULT);
	

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
	PUBLISH_ARG_NULL_CHECK(pub);
	z_drop(z_move(pub->session));
	return PUBLISH_OK;
}

publish_error_t publish_frame(publisher_t* pub, sensor_frame_t* frame) {
	PUBLISH_ARG_NULL_CHECK(pub);

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
