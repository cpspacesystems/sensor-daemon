#include <fcntl.h>
#include <stdint.h>
#include <stdlib.h>
#include <unistd.h>
#include <errno.h>
#include <sys/ioctl.h>

#include "i2c.h"
#include "linux-i2c-defs.h"
#include "linux-i2c-dev-defs.h"

#define I2C_CHECK_BUS_DEV(dev) { if (dev < 0) { return I2C_NO_DEVICE; } }
#define I2C_ARG_NULL_CHECK(arg) { if (!arg) { return I2C_INVALID_ARGUMENT; } }
#define I2C_CHECK_ALLOC(ptr) { if (!ptr) { return I2C_NO_MEM; } }
#define I2C_CHECK_DEVICE_FUNC(funcs, func) { if (!(funcs | func)) { return I2C_MISSING_FUNC; } }
#define I2C_MBIND(f) { i2c_error_t err = f; if (err != I2C_OK) { return err; } }

i2c_error_t i2c_bus_open(i2c_bus_handle_t* bus) {
	I2C_ARG_NULL_CHECK(bus);

	bus->fd = open("/dev/i2c-1", O_RDWR);

	if (bus->fd < 0) {
		goto handle_errno_open;
	}

	if (ioctl(bus->fd, I2C_FUNCS, &bus->funcs) < 0) {
		goto handle_errno_ioctl;
	}

	return I2C_OK;

handle_errno_open:
	switch (errno) {
		case EACCES: return I2C_BAD_PERMISSIONS;
		case EIO:    return I2C_IO_ERROR;
		case EMFILE:
		case ENFILE: return I2C_TOO_MANY_FDS;
		case ENOENT:
		case ENXIO:  return I2C_NO_DEVICE;
		case EINTR:  return I2C_INTERUPTED;
		default:     return I2C_UNKNOWN;
	}

handle_errno_ioctl:
	switch (errno) {
		case EBADF:
		case EINVAL: return I2C_INVALID_ARGUMENT;
		case ENOTTY: return I2C_NO_DEVICE;
		default:     return I2C_UNKNOWN;
	}
}

i2c_error_t i2c_bus_close(i2c_bus_handle_t* bus) {
	I2C_ARG_NULL_CHECK(bus);
	I2C_CHECK_BUS_DEV(bus->fd);

	if (close(bus->fd) < 0) {
		goto handle_errno;
	}

	return I2C_OK;

handle_errno:
	switch (errno) {
		case EBADF: return I2C_NO_DEVICE;
		case EIO:   return I2C_IO_ERROR;
		case EINTR: return I2C_INTERUPTED;
		default:    return I2C_UNKNOWN;
	}
}

i2c_error_t i2c_device_add(i2c_bus_handle_t* bus, i2c_addr_t dev) {
	return I2C_OK;
}

i2c_error_t i2c_device_set(i2c_bus_handle_t* bus, i2c_addr_t dev) {
	I2C_ARG_NULL_CHECK(bus);
	I2C_CHECK_BUS_DEV(bus->fd);

	if (ioctl(bus->fd, I2C_TENBIT, dev.width == I2C_ADDR_10_BIT) < 0) {
		goto handle_errno;
	}

	if (ioctl(bus->fd, I2C_SLAVE, dev.addr) < 0) {
		goto handle_errno;
	}

	return I2C_OK;

handle_errno:
	switch (errno) {
		case EBADF:
		case EINVAL: return I2C_INVALID_ARGUMENT;
		case ENOTTY: return I2C_NO_DEVICE;
		default:     return I2C_UNKNOWN;
	}
}

i2c_error_t i2c_write(i2c_bus_handle_t* bus, i2c_addr_t dev, uint8_t* buf, size_t n) {
	I2C_ARG_NULL_CHECK(bus);
	I2C_ARG_NULL_CHECK(buf);
	I2C_CHECK_BUS_DEV(bus->fd);

	I2C_MBIND(i2c_device_set(bus, dev));

	if (write(bus->fd, buf, n) != n) {
		goto handle_errno;
	}
	
	return I2C_OK;

handle_errno:
	switch (errno) {
		case EINVAL:
		case EBADF:  return I2C_INVALID_ARGUMENT;
		case EFBIG:
		case EIO:    return I2C_IO_ERROR;
		case EINTR:  return I2C_INTERUPTED;
		case ENOSPC: return I2C_NO_STORAGE;
		case ENXIO:  return I2C_NO_DEVICE;
		default:     return I2C_UNKNOWN;
	}
}

i2c_error_t i2c_write_byte(i2c_bus_handle_t* bus, i2c_addr_t dev, uint8_t byte) {
	return i2c_write(bus, dev, &byte, 1);
}

i2c_error_t i2c_read(i2c_bus_handle_t* bus, i2c_addr_t dev, uint8_t* buf, size_t n) {
	I2C_ARG_NULL_CHECK(bus);
	I2C_ARG_NULL_CHECK(buf);
	I2C_CHECK_BUS_DEV(bus->fd);

	I2C_MBIND(i2c_device_set(bus, dev));

	if (read(bus->fd, buf, n) != n) {
		goto handle_errno;
	}
	
	return I2C_OK;

handle_errno:
	switch (errno) {
		case EINVAL:
		case EBADF:  return I2C_INVALID_ARGUMENT;
		case EFBIG:
		case EIO:    return I2C_IO_ERROR;
		case EINTR:  return I2C_INTERUPTED;
		case ENOSPC: return I2C_NO_STORAGE;
		case ENXIO:  return I2C_NO_DEVICE;
		default:     return I2C_UNKNOWN;
	}
}

i2c_error_t i2c_read_byte(i2c_bus_handle_t* bus, i2c_addr_t dev, uint8_t* byte) {
	return i2c_read(bus, dev, byte, 1);
}

i2c_error_t i2c_write_read(i2c_bus_handle_t* bus, i2c_addr_t dev, uint8_t* tx_buf, size_t tx_n, uint8_t* rx_buf, size_t rx_n) {
	I2C_ARG_NULL_CHECK(tx_buf);
	I2C_ARG_NULL_CHECK(rx_buf);
	I2C_ARG_NULL_CHECK(bus);
	I2C_CHECK_BUS_DEV(bus->fd);
	I2C_CHECK_DEVICE_FUNC(bus->funcs, I2C_FUNC_I2C);

	uint16_t flags = 0;

	if (dev.width == I2C_ADDR_10_BIT) {
		I2C_CHECK_DEVICE_FUNC(bus->funcs, I2C_FUNC_10BIT_ADDR);
		flags |= I2C_M_TEN;
	}

	struct i2c_msg write = {
		.addr = dev.addr,
		.flags = flags,
		.buf = tx_buf,
		.len = tx_n
	};

	struct i2c_msg read = {
		.addr = dev.addr,
		.flags = flags | I2C_M_RD,
		.buf = rx_buf,
		.len = rx_n
	};
	
	struct i2c_msg msgs[2] = { write, read };
	struct i2c_rdwr_ioctl_data ioctl_data = {
		.msgs = msgs,
		.nmsgs = 2,
	};

	if (ioctl(bus->fd, I2C_RDWR, &ioctl_data) < 0) {
		goto handle_errno;
	}

	return I2C_OK;

handle_errno:
	switch (errno) {
		case EBADF:
		case EINVAL: return I2C_INVALID_ARGUMENT;
		case ENOTTY: return I2C_NO_DEVICE;
		default:     return I2C_UNKNOWN;
	}
}

i2c_error_t i2c_register_read(i2c_bus_handle_t* bus, i2c_addr_t dev, uint8_t reg, uint8_t* data) {
	return i2c_write_read(bus, dev, &reg, 1, data, 1);
}
