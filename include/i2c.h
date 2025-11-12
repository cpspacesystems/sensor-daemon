/**
 * Wrapper over Linux I2C functionality.
 */

#ifndef _I2C_H
#define _I2C_H

#include <stdio.h>
#include <stdint.h>

/**
 * Like most errors "OK" is zero, errors are negative.
 */
typedef enum {
	/**
	 * No error.
	 */
	I2C_OK = 0,

	/**
	 * This one's on you buddy :3
	 */
	I2C_INVALID_ARGUMENT = -1,

	/**
	 * You're not allowed to do that lmao
	 */
	I2C_BAD_PERMISSIONS = -2,

	/**
	 * I/O error. Blame a solar flair or something.
	 */
	I2C_IO_ERROR = -3,

	/**
	 * To many files are open, either for this process or the system as a whole.
	 */
	I2C_TOO_MANY_FDS = -4,

	/**
	 * Required I2C device does not exist. This error is also given is a bad file
	 * descriptor is given to any function which intends to perform I/O on the I2C
	 * bus, which is just about everything except `i2c_bus_open`.
	 *
	 * This error may also be given if a device's setup was otherwise successful,
	 * but even so the device is proveably not present or working correctly, i.e.
	 * if a register which should give a known value does not produce that value
	 * correctly. As such this error may be returned by drivers using I2C to
	 * represent the failure of certain checks they might perform over the bus.
	 */
	I2C_NO_DEVICE = -5,

	/**
	 * I have no memory but I must `malloc` :(
	 */
	I2C_NO_MEM = -6,

	/**
	 * I have no storage but I must `write` :(
	 */
	I2C_NO_STORAGE = -7,

	/**
	 * SOME FUCKER CUT ME OFF!!!
	 *
	 * (write/read interrupted by signal).
	 */
	I2C_INTERUPTED = -8,

	/**
	 * Missing functionality in report from `ioctl`. This should be taken to mean
	 * that the linux kernel has deemed your I2C bus unworthy of the feature you
	 * wish to use.
	 */
	I2C_MISSING_FUNC = -9,

	/**
	 * Unknown error.
	 */
	I2C_UNKNOWN = -128
} i2c_error_t;

/**
 * Width of I2C device's address.
 */
typedef enum {
	/**
	 * Seven bit address.
	 */
	I2C_ADDR_7_BIT = 7,

	/**
	 * Ten bit address. Note that the linux driver/API for I2C doesn't properly
	 * support ten bit addresses, so using this option is probably just a bad idea.
	 */
	I2C_ADDR_10_BIT = 10,
} i2c_addr_width_t;

/**
 * Device address on an I2C bus.
 */
typedef struct {
	/**
	 * Width of the address, either seven or ten bits.
	 */
	i2c_addr_width_t width;

	/**
	 * Actual device address. Do not shift this over or include the read/write bit,
	 * etc. This should be the actual numeric address as it is documented by the
	 * device you wish to target.
	 */
	uint16_t addr;
} i2c_addr_t;

/**
 * Type definition over an `int` file descriptor meant for holding a handle to
 * the local I2C device, which is our access to the bus as a whole.
 *
 * Refered to as the whole bus rather than the single device it actually is to
 * avoid confusion with the slave devices which we interact with through this
 * "bus" or master device.
 */
typedef struct {
	/** Funcs as got by an `I2C_FUNCS` `ioctl` call. */
	unsigned long funcs;

	/** File descriptor. */
	int fd;
} i2c_bus_handle_t;

/**
 * Open the I2C device. Populates the given `i2c_bus_handle_t` with a file
 * descriptor of the in-use I2C device.
 *
 * This function will select one of "/dev/i2c-0" or "/dev/i2c-1" depending on
 * the GPIO layout of the Raspberry Pi model being used, so essentially
 * depending on the model of Pi.
 */
i2c_error_t i2c_bus_open(i2c_bus_handle_t* bus);

/**
 * Close the given I2C bus.
 */
i2c_error_t i2c_bus_close(i2c_bus_handle_t* bus);

/**
 * Add/setup an I2C device at the given `i2c_addr_t` on the given I2C bus.
 */
i2c_error_t i2c_device_add(i2c_bus_handle_t* bus, i2c_addr_t dev);

/**
 * Set the current I2C device. This makes it so that `read`/`write` calls using
 * the given `bus` file descriptor will go to the address given. This is
 * probably not something you want to use, unless you wish to perform I/O
 * without the use of this interface.
 */
i2c_error_t i2c_device_set(i2c_bus_handle_t* bus, i2c_addr_t dev);

/**
 * Write `n` bytes over the given `bus` to the device at the given `i2c_addr_t`.
 */
i2c_error_t i2c_write(i2c_bus_handle_t* bus, i2c_addr_t dev, uint8_t* buf, size_t n);

/**
 * Write a single byte over I2C to the given device.
 */
i2c_error_t i2c_write_byte(i2c_bus_handle_t* bus, i2c_addr_t dev, uint8_t byte);

/**
 * Read out `n` bytes over I2C into the given `buf`. This function considers it
 * an error to read out less than `n` bytes over I2C.
 */
i2c_error_t i2c_read(i2c_bus_handle_t* bus, i2c_addr_t dev, uint8_t* buf, size_t n);

/**
 * Read a single byte over I2C from the given device.
 */
i2c_error_t i2c_read_byte(i2c_bus_handle_t* bus, i2c_addr_t dev, uint8_t* byte);

/**
 * Write the given `tx_bus` of length `tx_n` to the given `i2c_addr_t` and then,
 * in a single operation/transaction, read out `rx_n` bytes from that device
 * into `rx_buf`.
 */
i2c_error_t i2c_write_read(i2c_bus_handle_t* bus, i2c_addr_t dev, uint8_t* tx_buf, size_t tx_n, uint8_t* rx_buf, size_t rx_n);

/**
 * Intended for single byte named registers which contains single byte data.
 *
 * Writes to the device a single byte, and reads back a single byte as well.
 */
i2c_error_t i2c_register_read(i2c_bus_handle_t* bus, i2c_addr_t dev, uint8_t reg, uint8_t* data);

#endif  /* _I2C_H */
