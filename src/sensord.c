#include <errno.h>
#include <fcntl.h>
#include <stdio.h>
#include <string.h>
#include <sys/mman.h>
#include <unistd.h>

#include "sensord.h"
#include "config.h"
#include "log.h"
#include "publish.h"
#include "sensor_frame.h"

#define SENSORD_CHECK_ARG_NULL(arg) if (!arg) return PUBLISH_INVALID_ARGUMENT;

sensord_error_t sensord_init(sensord_reciever_t* recv) {
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

static void sensord_get_callback(z_loaned_reply_t* reply, void* ctx) {
	sensor_frame_t* frame = (sensor_frame_t*)ctx;

	if (!z_reply_is_ok(reply)) {
		LOG_ERROR("Reply was not ok!");
		return;
	}

	const z_loaned_sample_t* sample = z_reply_ok(reply);
	const z_loaned_bytes_t* bytes = z_sample_payload(sample);

	z_bytes_reader_t reader = z_bytes_get_reader(bytes);
	size_t n = z_bytes_reader_read(&reader, (uint8_t*)frame, sizeof *frame);

	if (n != sizeof *frame) {
		LOG_ERROR("Reply did not contain correct number of bytes (expected %u, got %u)!", sizeof *frame, n);
	}

	LOG_DEBUG("Zenoh get callback returned.");
}

sensord_error_t sensord_get_sensor_frame(sensord_reciever_t* recv, sensor_frame_t* frame) {
	z_result_t z_res;
	z_get_options_t opts;
	z_owned_closure_reply_t get_callback;

	z_get_options_default(&opts);
	z_closure(&get_callback, sensord_get_callback, NULL, (void*)frame);

	if ((z_res = z_get(z_loan(recv->session), z_loan(recv->keyexpr), "", z_move(get_callback), &opts)) != Z_OK) {
		LOG_ERROR("Could not get sensor frame (%i)!", z_res);
		return PUBLISH_Z_GET;
	}

	sleep(1);

	return PUBLISH_OK;
}

sensord_error_t sensord_cleanup(sensord_reciever_t* recv) {
	SENSORD_CHECK_ARG_NULL(recv);
	z_drop(z_move(recv->session));
	return PUBLISH_OK;
}
