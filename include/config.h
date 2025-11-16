/*
 * Some basic configuration options. These are much less preferences than they
 * are semi-arbitrary constants which may be tweaked.
 */

#ifndef _CONFIG_H
#define _CONFIG_H

/**
 * Maximum number of retries which are permissable for adding a device to the
 * I2C bus.
 */
#define MAX_I2C_DEVICE_ADD_RETRIES 10

/**
 * Maximum number of retries for opening a Zenoh session. Feel free to make this
 * fairly high, since Zenoh sessions can fail to open if there isnt a Zenoh
 * daemon instance to connect to, and we would like for the sensor daemon to
 * wait for a Zenoh daemon instance.
 */
#define MAX_ZENOH_SESSION_OPEN_RETRIES 16

/**
 * Port which Zenoh should use, assuming Zenoh is enabled (should be a string
 * literal).
 */
#define ZENOH_TCP_LOCATOR_PORT "7447"

/**
 * Key for putting new sensor frames over Zenoh.
 */
#define ZENOH_PUT_KEY "cpss/sensor_frame"

#endif  /** _CONFIG_H */
