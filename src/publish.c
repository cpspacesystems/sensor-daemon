#include <errno.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include "publish.h"
#include "sensor_frame.h"
#include "log.h"
#include "config.h"

#include <zenoh-pico.h>

#define PUBLISH_ARG_NULL_CHECK(ptr) if (!ptr) return PUBLISH_INVALID_ARGUMENT;

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
