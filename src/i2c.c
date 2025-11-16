#include <fcntl.h>
#include <stdint.h>
#include <stdlib.h>
#include <unistd.h>
#include <errno.h>
#include <sys/ioctl.h>
#include <sys/stat.h>

#include "i2c.h"
#include "linux-i2c-defs.h"
#include "linux-i2c-dev-defs.h"
#include "log.h"

#define I2C_CHECK_BUS_DEV(dev) { if (dev < 0) { return I2C_NO_DEVICE; } }
#define I2C_ARG_NULL_CHECK(arg) { if (!arg) { return I2C_INVALID_ARGUMENT; } }
#define I2C_CHECK_ALLOC(ptr) { if (!ptr) { return I2C_NO_MEM; } }
#define I2C_MBIND(f) { i2c_error_t err = f; if (err != I2C_OK) { return err; } }
#define I2C_CHECK_DEV(dev) \
	if (dev.width == I2C_ADDR_10_BIT && dev.addr < 0x0400) { \
		return I2C_INVALID_ARGUMENT; \
	} else if (dev.width == I2C_ADDR_7_BIT && dev.addr < 0x0080) { \
		return I2C_INVALID_ARGUMENT; \
	}

i2c_error_t i2c_bus_auto_find(char* device) {
	struct stat stat_res;

	for (uint16_t i2c_device = 0; i2c_device < 256; i2c_device++) {
		sprintf(device, "/dev/i2c-%u", i2c_device);

		if (stat(device, &stat_res) == 0) {
			return I2C_OK;
		}
	}

	return I2C_NO_BUS;
}

i2c_error_t i2c_bus_open(i2c_bus_handle_t* bus, const char* device) {
	I2C_ARG_NULL_CHECK(bus);

	bus->fd = open(device, O_RDWR);

	if (bus->fd < 0) {
		LOG_ERROR("Could not open I2C bus!");
		goto handle_errno_open;
	}

	if (ioctl(bus->fd, I2C_FUNCS, &bus->funcs) < 0) {
		LOG_ERROR("Could get I2C functionality!");
		goto handle_errno_ioctl;
	}

	LOG_INFO("Succesfully opened I2C bus.");
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
	I2C_ARG_NULL_CHECK(bus);
	I2C_CHECK_BUS_DEV(bus->fd);
	I2C_CHECK_DEV(dev);

	if (ioctl(bus->fd, I2C_SLAVE, dev.addr) < 0) {
		if (dev.width == I2C_ADDR_10_BIT) {
			LOG_ERROR("Failed to set slave device at address '%#002x'!", dev.addr);
		} else {
			LOG_ERROR("Failed to set slave device at address '%#02x'!", dev.addr);
		}

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

i2c_error_t i2c_device_set(i2c_bus_handle_t* bus, i2c_addr_t dev) {
	I2C_ARG_NULL_CHECK(bus);
	I2C_CHECK_BUS_DEV(bus->fd);
	I2C_CHECK_DEV(dev);

	if (dev.width == I2C_ADDR_10_BIT && !(bus->funcs & I2C_FUNC_10BIT_ADDR)) {
		LOG_ERROR("Attempted usage of 10 bit address but I2C bus lacked required functionality!");
		return I2C_MISSING_FUNC;
	}

	if (ioctl(bus->fd, I2C_TENBIT, dev.width == I2C_ADDR_10_BIT) < 0) {
		LOG_ERROR("Failed to set I2C address width to 10 bit!");
		goto handle_errno;
	}

	if (ioctl(bus->fd, I2C_SLAVE, dev.addr) < 0) {
		if (dev.width == I2C_ADDR_10_BIT) {
			LOG_ERROR("Failed to set slave device at address '%#002x'!", dev.addr);
		} else {
			LOG_ERROR("Failed to set slave device at address '%#02x'!", dev.addr);
		}

		goto handle_errno;
	}

	if (dev.width == I2C_ADDR_10_BIT) {
		LOG_DEBUG("Set to use device at address '%#002x' over I2C bus.", dev.addr);
	} else {
		LOG_DEBUG("Set to use device at address '%#02x' over I2C bus.", dev.addr);
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
	I2C_CHECK_DEV(dev);

	I2C_MBIND(i2c_device_set(bus, dev));

	if (write(bus->fd, buf, n) != n) {
		LOG_ERROR("Failed write operation over I2C!");
		goto handle_errno;
	}
	
	LOG_DEBUG("Wrote %u bytes over I2C.", n);
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
	I2C_CHECK_DEV(dev);

	I2C_MBIND(i2c_device_set(bus, dev));

	if (read(bus->fd, buf, n) != n) {
		LOG_ERROR("Failed read operation over I2C!");
		goto handle_errno;
	}
	
	LOG_DEBUG("Read %u bytes over I2C.", n);
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
	I2C_CHECK_DEV(dev);

	if (!(bus->funcs & I2C_FUNC_I2C)) {
		LOG_ERROR("Attempted combined read/write operation but I2C bus lacked required functionality!");
		return I2C_MISSING_FUNC;
	}

	uint16_t flags = 0;

	if (dev.width == I2C_ADDR_10_BIT) {
		if (!(bus->funcs & I2C_FUNC_10BIT_ADDR)) {
			LOG_ERROR("Attempted usage of 10 bit address but I2C bus lacked required functionality!");
			return I2C_MISSING_FUNC;
		}

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
		LOG_ERROR("Failed combined read/write operation over I2C!");
		goto handle_errno;
	}

	LOG_DEBUG("Performed combined write (%u bytes) then read (%u bytes) over I2C.", tx_n, rx_n);
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
