#include <stdint.h>

#include "i2c.h"
#include "log.h"
#include "drivers/icm_20948.h"

#define ICM_ERROR_MBIND(e) { i2c_error_t err = e; if (err != I2C_OK) { return err; } }

i2c_error_t icm_setup(i2c_bus_handle_t* bus, i2c_addr_t* dev, bool ad0) {
	dev->addr = ICM_I2C_ADDR;
	dev->width = I2C_ADDR_7_BIT;

	if (ad0) {
		dev->addr |= ICM_I2C_ADDR_AD0_HIGH;
	}
	
	ICM_ERROR_MBIND(i2c_device_add(bus, *dev));

	uint8_t who_am_i = 0;
	ICM_ERROR_MBIND(i2c_register_read(bus, *dev, ICM_WHO_AM_I, &who_am_i));

	if (who_am_i != ICM_WHO_AM_I_RST) {
		LOG_ERROR("Added IMU to the I2C bus, but the `WHO_AM_I` register was bad!");
		return I2C_NO_DEVICE;
	}
	
	LOG_INFO("Added IMU to the I2C bus succesfully.");
	return I2C_OK;
}

i2c_error_t icm_register_read_word(
	i2c_bus_handle_t* bus,
	i2c_addr_t dev,
	icm_register_t reg_high,
	icm_register_t reg_low,
	uint16_t* data
) {
	uint8_t high_byte = 0;
	uint8_t low_byte = 0;

	ICM_ERROR_MBIND(i2c_register_read(bus, dev, reg_high, &high_byte));
	ICM_ERROR_MBIND(i2c_register_read(bus, dev, reg_low, &low_byte));

	*data = (high_byte << 8) | low_byte;
	return I2C_OK;
}

i2c_error_t icm_register_write(i2c_bus_handle_t* bus, i2c_addr_t dev, icm_register_t reg, uint8_t data) {
	uint8_t tx[2] = { reg, data };
	ICM_ERROR_MBIND(i2c_write(bus, dev, tx, 2));
	return I2C_OK;
}

i2c_error_t icm_register_write_word(
	i2c_bus_handle_t* bus,
	i2c_addr_t dev,
	icm_register_t reg_high,
	icm_register_t reg_low,
	uint16_t data
) {
	uint8_t high_byte = (data & 0xFF00) >> 8;
	uint8_t low_byte = data & 0x00FF;

	ICM_ERROR_MBIND(icm_register_write(bus, dev, reg_high, high_byte));
	ICM_ERROR_MBIND(icm_register_write(bus, dev, reg_low, low_byte));

	return I2C_OK;
}

i2c_error_t icm_gyro_out_x(i2c_bus_handle_t* bus, i2c_addr_t dev, uint16_t* data) {
	ICM_ERROR_MBIND(icm_register_read_word(bus, dev, ICM_GYRO_XOUT_H, ICM_GYRO_XOUT_L, data));
	return I2C_OK;
}

i2c_error_t icm_gyro_out_y(i2c_bus_handle_t* bus, i2c_addr_t dev, uint16_t* data) {
	ICM_ERROR_MBIND(icm_register_read_word(bus, dev, ICM_GYRO_YOUT_H, ICM_GYRO_YOUT_L, data));
	return I2C_OK;
}

i2c_error_t icm_gyro_out_z(i2c_bus_handle_t* bus, i2c_addr_t dev, uint16_t* data) {
	ICM_ERROR_MBIND(icm_register_read_word(bus, dev, ICM_GYRO_ZOUT_H, ICM_GYRO_ZOUT_L, data));
	return I2C_OK;
}

i2c_error_t icm_accel_out_x(i2c_bus_handle_t* bus, i2c_addr_t dev, uint16_t* data) {
	ICM_ERROR_MBIND(icm_register_read_word(bus, dev, ICM_ACCEL_XOUT_H, ICM_ACCEL_XOUT_L, data));
	return I2C_OK;
}

i2c_error_t icm_accel_out_y(i2c_bus_handle_t* bus, i2c_addr_t dev, uint16_t* data) {
	ICM_ERROR_MBIND(icm_register_read_word(bus, dev, ICM_ACCEL_YOUT_H, ICM_ACCEL_YOUT_L, data));
	return I2C_OK;
}

i2c_error_t icm_accel_out_z(i2c_bus_handle_t* bus, i2c_addr_t dev, uint16_t* data) {
	ICM_ERROR_MBIND(icm_register_read_word(bus, dev, ICM_ACCEL_ZOUT_H, ICM_ACCEL_ZOUT_L, data));
	return I2C_OK;
}

i2c_error_t icm_temp(i2c_bus_handle_t* bus, i2c_addr_t dev, uint16_t* data) {
	ICM_ERROR_MBIND(icm_register_read_word(bus, dev, ICM_TEMP_OUT_H, ICM_TEMP_OUT_L, data));
	return I2C_OK;
}

i2c_error_t icm_get_gyro_x(i2c_bus_handle_t* bus, i2c_addr_t dev, double* data) {
	uint16_t raw = 0;
	ICM_ERROR_MBIND(icm_gyro_out_x(bus, dev, &raw));
	*data = (double)raw / (double)ICM_GYRO_SENSITIVITY;
	return I2C_OK;
}

i2c_error_t icm_get_gyro_y(i2c_bus_handle_t* bus, i2c_addr_t dev, double* data) {
	uint16_t raw = 0;
	ICM_ERROR_MBIND(icm_gyro_out_y(bus, dev, &raw));
	*data = (double)raw / (double)ICM_GYRO_SENSITIVITY;
	return I2C_OK;
}

i2c_error_t icm_get_gyro_z(i2c_bus_handle_t* bus, i2c_addr_t dev, double* data) {
	uint16_t raw = 0;
	ICM_ERROR_MBIND(icm_gyro_out_z(bus, dev, &raw));
	*data = (double)raw / (double)ICM_GYRO_SENSITIVITY;
	return I2C_OK;
}


i2c_error_t icm_get_gyro_x_f(i2c_bus_handle_t* bus, i2c_addr_t dev, float* data) {
	uint16_t raw = 0;
	ICM_ERROR_MBIND(icm_gyro_out_x(bus, dev, &raw));
	*data = (float)raw / (float)ICM_GYRO_SENSITIVITY;
	return I2C_OK;
}

i2c_error_t icm_get_gyro_y_f(i2c_bus_handle_t* bus, i2c_addr_t dev, float* data) {
	uint16_t raw = 0;
	ICM_ERROR_MBIND(icm_gyro_out_y(bus, dev, &raw));
	*data = (float)raw / (float)ICM_GYRO_SENSITIVITY;
	return I2C_OK;
}

i2c_error_t icm_get_gyro_z_f(i2c_bus_handle_t* bus, i2c_addr_t dev, float* data) {
	uint16_t raw = 0;
	ICM_ERROR_MBIND(icm_gyro_out_z(bus, dev, &raw));
	*data = (float)raw / (float)ICM_GYRO_SENSITIVITY;
	return I2C_OK;
}

i2c_error_t icm_get_accel_x(i2c_bus_handle_t* bus, i2c_addr_t dev, double* data) {
	uint16_t raw = 0;
	ICM_ERROR_MBIND(icm_accel_out_x(bus, dev, &raw));
	*data = (double)raw / (double)ICM_ACCELEROMETER_SENSITIVITY;
	return I2C_OK;
}

i2c_error_t icm_get_accel_y(i2c_bus_handle_t* bus, i2c_addr_t dev, double* data) {
	uint16_t raw = 0;
	ICM_ERROR_MBIND(icm_accel_out_y(bus, dev, &raw));
	*data = (double)raw / (double)ICM_ACCELEROMETER_SENSITIVITY;
	return I2C_OK;
}

i2c_error_t icm_get_accel_z(i2c_bus_handle_t* bus, i2c_addr_t dev, double* data) {
	uint16_t raw = 0;
	ICM_ERROR_MBIND(icm_accel_out_z(bus, dev, &raw));
	*data = (double)raw / (double)ICM_ACCELEROMETER_SENSITIVITY;
	return I2C_OK;
}

i2c_error_t icm_get_accel_x_f(i2c_bus_handle_t* bus, i2c_addr_t dev, float* data) {
	uint16_t raw = 0;
	ICM_ERROR_MBIND(icm_accel_out_x(bus, dev, &raw));
	*data = (float)raw / (float)ICM_ACCELEROMETER_SENSITIVITY;
	return I2C_OK;
}

i2c_error_t icm_get_accel_y_f(i2c_bus_handle_t* bus, i2c_addr_t dev, float* data) {
	uint16_t raw = 0;
	ICM_ERROR_MBIND(icm_accel_out_y(bus, dev, &raw));
	*data = (float)raw / (float)ICM_ACCELEROMETER_SENSITIVITY;
	return I2C_OK;
}

i2c_error_t icm_get_accel_z_f(i2c_bus_handle_t* bus, i2c_addr_t dev, float* data) {
	uint16_t raw = 0;
	ICM_ERROR_MBIND(icm_accel_out_z(bus, dev, &raw));
	*data = (float)raw / (float)ICM_ACCELEROMETER_SENSITIVITY;
	return I2C_OK;
}

i2c_error_t icm_get_temp(i2c_bus_handle_t* bus, i2c_addr_t dev, double* data) {
	uint16_t raw = 0;
	ICM_ERROR_MBIND(icm_temp(bus, dev, &raw));
	*data = 21.0 + (double)(raw - ICM_ROOM_TEMP_OFFSET) / (double)ICM_TEMP_SENSITIVITY;
	return I2C_OK;
}

i2c_error_t icm_get_temp_f(i2c_bus_handle_t* bus, i2c_addr_t dev, float* data) {
	uint16_t raw = 0;
	ICM_ERROR_MBIND(icm_temp(bus, dev, &raw));
	*data = 21.0 + (float)(raw - ICM_ROOM_TEMP_OFFSET) / (float)ICM_TEMP_SENSITIVITY;
	return I2C_OK;
}
