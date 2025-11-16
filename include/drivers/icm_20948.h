/*
 * I2C based driver for the ICM 20948 9 DoF IMU.
 */

#ifndef _ICM_20948
#define _ICM_20948

/**
 * This driver/library/API will assume that I2C is set up in the capacity that
 * it must on the system. The addition of the ICM 20948 IMU device on the I2C
 * bus however will not be assumed, and a function to add it is provided.
 *
 * Since its a long name this header will prefix types and functions with
 * "icm_", rather than the full title.
 *
 * This mini-API will use a few naming conventions for ease of use. Any "high
 * level" function, which may perform some kind of conversion or otherwise gives
 * human readable values, will be an "icm_get_" function. `double` will be
 * considered the default floating point type but function which deal in `float`
 * are provided and are suffixed with "_f".
 */

#include <stdint.h>
#include <stdbool.h>

#include "i2c.h"

/**
 * The base address of the IMU on I2C, the last bit is set by the logic level of
 * the AD0 pin, allowing for a choice of address in the least significant bit
 * only.
 */
#define ICM_I2C_ADDR 0b1101000

/**
 * Component of the I2C address determined by the AD0 pin on the IMU. This
 * constant should be added/bitwise-or'd into the base address if AD0 is set
 * high.
 */
#define ICM_I2C_ADDR_AD0_HIGH 0b0000001

/**
 * "Reset" value of the `ICM_WHO_AM_I` register. This should always be the value
 * in that register.
 */
#define ICM_WHO_AM_I_RST 0xEA

/**
 * Conversion for raw gyro measurements to a usable value (in no particular
 * unit). Measurement will be divided by this constant in higher level functions
 * which return floating point values.
 */
#define ICM_GYRO_SENSITIVITY 0xFFFF

/**
 * Conversion for raw accel measurements to a usable value (in no particular
 * unit). Measurement will be divided by this constant in higher level functions
 * which return floating point values.
 */
#define ICM_ACCELEROMETER_SENSITIVITY 0xFFFF

/*
 * (TEMP_OUT - ICM_ROOM_TEMP_OFFSET) / ICM_TEMP_SENSITIVITY + 21degC = temp
 * Temps seem to be celsius by default here, unlike the other guys which dont
 * specify units.
 */
#define ICM_ROOM_TEMP_OFFSET 0xFFFF
#define ICM_TEMP_SENSITIVITY 0xFFFF

/**
 * Registers available for the ICM 20948. Not all are writeable but all are
 * readable. These are used to access data over I2C.
 *
 * Registers worry only about a single byte of data at a time here, hence the
 * commonality of high/low bytes register pairs.
 */
typedef enum {
	/* All user banks */

	ICM_REG_BANK_SEL = 0x7F,

	/* User bank 0 registers */
	
	ICM_WHO_AM_I = 0x00,

	ICM_USER_CTRL = 0x03,

	ICM_LP_CONFIG = 0x05,

	ICM_PWR_MGMT_1 = 0x06,
	ICM_PWR_MGMT_2 = 0x07,

	ICM_INT_PIN_CFG = 0x0F,

	ICM_INT_ENABLE = 0x10,

	ICM_INT_ENABLE_1 = 0x11,
	ICM_INT_ENABLE_2 = 0x12,
	ICM_INT_ENABLE_3 = 0x12,

	ICM_I2C_MST_STATUS = 0x17,

	ICM_INT_STATUS = 0x19,

	ICM_INT_STATUS_1 = 0x1A,
	ICM_INT_STATUS_2 = 0x1B,
	ICM_INT_STATUS_3 = 0x1C,

	ICM_DELAY_TIMEH = 0x28,
	ICM_DELAY_TIMEL = 0x29,

	ICM_ACCEL_XOUT_H = 0x2D,
	ICM_ACCEL_XOUT_L = 0x2E,
	ICM_ACCEL_YOUT_H = 0x2F,
	ICM_ACCEL_YOUT_L = 0x30,
	ICM_ACCEL_ZOUT_H = 0x31,
	ICM_ACCEL_ZOUT_L = 0x32,

	ICM_GYRO_XOUT_H = 0x33,
	ICM_GYRO_XOUT_L = 0x34,
	ICM_GYRO_YOUT_H = 0x35,
	ICM_GYRO_YOUT_L = 0x36,
	ICM_GYRO_ZOUT_H = 0x37,
	ICM_GYRO_ZOUT_L = 0x38,

	ICM_TEMP_OUT_H = 0x39,
	ICM_TEMP_OUT_L = 0x3A,

	ICM_EXT_SLV_SENS_DATA_00 = 0x3B,
	ICM_EXT_SLV_SENS_DATA_01 = 0x3C,
	ICM_EXT_SLV_SENS_DATA_02 = 0x3D,
	ICM_EXT_SLV_SENS_DATA_03 = 0x3E,
	ICM_EXT_SLV_SENS_DATA_04 = 0x3F,
	ICM_EXT_SLV_SENS_DATA_05 = 0x40,
	ICM_EXT_SLV_SENS_DATA_06 = 0x41,
	ICM_EXT_SLV_SENS_DATA_07 = 0x42,
	ICM_EXT_SLV_SENS_DATA_08 = 0x43,
	ICM_EXT_SLV_SENS_DATA_09 = 0x44,
	ICM_EXT_SLV_SENS_DATA_10 = 0x45,
	ICM_EXT_SLV_SENS_DATA_11 = 0x46,
	ICM_EXT_SLV_SENS_DATA_12 = 0x47,
	ICM_EXT_SLV_SENS_DATA_13 = 0x48,
	ICM_EXT_SLV_SENS_DATA_14 = 0x49,
	ICM_EXT_SLV_SENS_DATA_15 = 0x4A,
	ICM_EXT_SLV_SENS_DATA_16 = 0x4B,
	ICM_EXT_SLV_SENS_DATA_17 = 0x4C,
	ICM_EXT_SLV_SENS_DATA_18 = 0x4D,
	ICM_EXT_SLV_SENS_DATA_19 = 0x4E,
	ICM_EXT_SLV_SENS_DATA_20 = 0x4F,
	ICM_EXT_SLV_SENS_DATA_21 = 0x50,
	ICM_EXT_SLV_SENS_DATA_22 = 0x51,
	ICM_EXT_SLV_SENS_DATA_23 = 0x52,

	ICM_FIFO_EN_1 = 0x66,
	ICM_FIFO_EN_2 = 0x67,
	ICM_FIFO_RST = 0x68,
	ICM_FIFO_MODE = 0x69,

	ICM_FIFO_COUNTH = 0x70,
	ICM_FIFO_COUNTL = 0x71,
	ICM_FIFO_R_W = 0x72,

	ICM_DATA_RDY_STATUS = 0x74,

	ICM_FIFO_CFG = 0x76,

	/* User bank 1 registers */

	ICM_SELF_TEST_X_GYRO = 0x02,
	ICM_SELF_TEST_Y_GYRO = 0x03,
	ICM_SELF_TEST_Z_GYRO = 0x04,

	ICM_SELF_TEST_X_ACCEL = 0x0E,
	ICM_SELF_TEST_Y_ACCEL = 0x0F,
	ICM_SELF_TEST_Z_ACCEL = 0x10,

	ICM_XA_OFFS_H = 0x14,
	ICM_XA_OFFS_L = 0x15,

	ICM_YA_OFFS_H = 0x17,
	ICM_YA_OFFS_L = 0x18,

	ICM_ZA_OFFS_H = 0x1A,
	ICM_ZA_OFFS_L = 0x1B,

	ICM_TIMEBASE_CORRECTION_PLL = 0x28,

	/* User bank 2 registers */

	IMC_GYRO_SMPLRT_DIV = 0x00,

	IMC_GYRO_CONFIG_1 = 0x01,
	IMC_GYRO_CONFIG_2 = 0x02,

	IMC_XG_OFFS_USRH = 0x03,
	IMC_XG_OFFS_USRL = 0x04,
	
	IMC_YG_OFFS_USRH = 0x05,
	IMC_YG_OFFS_USRL = 0x06,
	
	IMC_ZG_OFFS_USRH = 0x07,
	IMC_ZG_OFFS_USRL = 0x08,

	IMC_ODR_ALIGN_EN = 0x09,

	IMC_ACCEL_SMPLRT_DIV_1 = 0x10,
	IMC_ACCEL_SMPLRT_DIV_2 = 0x11,

	IMC_INTEL_CTRL = 0x12,

	IMC_ACCEL_WOM_THR = 0x13,
	IMC_ACCEL_CONFIG = 0x14,
	IMC_ACCEL_CONFIG_2 = 0x15,

	IMC_FSYNC_CONFIG = 0x52,
	IMC_TEMP_CONFIG = 0x53,

	IMC_MOD_CTRL_USR = 0x54,
	
	/* User bank 3 registers */

	IMC_I2C_MST_ODR_CONFIG = 0x00,
	IMC_I2C_MST_CTRL = 0x01,
	IMC_I2C_MST_DELAY_CTRL = 0x02,

	IMC_I2C_SLV0_ADDR = 0x03,
	IMC_I2C_SLV0_REG = 0x04,
	IMC_I2C_SLV0_CTRL = 0x05,
	IMC_I2C_SLV0_DO = 0x06,

	IMC_I2C_SLV1_ADDR = 0x07,
	IMC_I2C_SLV1_REG = 0x08,
	IMC_I2C_SLV1_CTRL = 0x09,
	IMC_I2C_SLV1_DO = 0x0A,

	IMC_I2C_SLV2_ADDR = 0x0B,
	IMC_I2C_SLV2_REG = 0x0C,
	IMC_I2C_SLV2_CTRL = 0x0D,
	IMC_I2C_SLV2_DO = 0x0E,

	IMC_I2C_SLV3_ADDR = 0x0F,
	IMC_I2C_SLV3_REG = 0x10,
	IMC_I2C_SLV3_CTRL = 0x11,
	IMC_I2C_SLV3_DO = 0x12,

	IMC_I2C_SLV4_ADDR = 0x13,
	IMC_I2C_SLV4_REG = 0x14,
	IMC_I2C_SLV4_CTRL = 0x15,
	IMC_I2C_SLV4_DO = 0x16,
	IMC_I2C_SLV4_DI = 0x17,
} icm_register_t;

/**
 * Setup the ICM 20948 on the I2C bus. This function takes an initialized I2C
 * bus and an uninitialized `i2c_addr_t`, which will be initialized by this
 * function with appropriate values. If the IMU's AD0 pin is set high this
 * function should be passed `true` for `ad0`, otherwise `false`.
 */
i2c_error_t icm_setup(i2c_bus_handle_t* bus, i2c_addr_t* dev, bool ad0);

/**
 * Read two registers, combining their results into a single `uint16_t`. The
 * high byte of `data` will be filled with data from the `reg_high` register,
 * and the low byte will be filled with data from the `reg_low` register.
 */
i2c_error_t icm_register_read_word(
	i2c_bus_handle_t* bus,
	i2c_addr_t dev,
	icm_register_t reg_high,
	icm_register_t reg_low,
	uint16_t* data
);

/**
 * Write the given `uint8_t` to an ICM register.
 */
i2c_error_t icm_register_write(
	i2c_bus_handle_t* bus,
	i2c_addr_t dev,
	icm_register_t reg,
	uint8_t data
);

/**
 * Write to two registers, parsing their individual values from a single
 * `uint16_t`. The high byte of `data` will fill the `reg_high` register, and
 * the low byte will fill the `reg_low` register.
 */
i2c_error_t icm_register_write_word(
	i2c_bus_handle_t* bus,
	i2c_addr_t dev,
	icm_register_t reg_high,
	icm_register_t reg_low,
	uint16_t data
);

i2c_error_t icm_gyro_out_x(i2c_bus_handle_t* bus, i2c_addr_t dev, uint16_t* data);
i2c_error_t icm_gyro_out_y(i2c_bus_handle_t* bus, i2c_addr_t dev, uint16_t* data);
i2c_error_t icm_gyro_out_z(i2c_bus_handle_t* bus, i2c_addr_t dev, uint16_t* data);

i2c_error_t icm_accel_out_x(i2c_bus_handle_t* bus, i2c_addr_t dev, uint16_t* data);
i2c_error_t icm_accel_out_y(i2c_bus_handle_t* bus, i2c_addr_t dev, uint16_t* data);
i2c_error_t icm_accel_out_z(i2c_bus_handle_t* bus, i2c_addr_t dev, uint16_t* data);

i2c_error_t icm_temp(i2c_bus_handle_t* bus, i2c_addr_t dev, uint16_t* data);

i2c_error_t icm_get_gyro_x(i2c_bus_handle_t* bus, i2c_addr_t dev, double* data);
i2c_error_t icm_get_gyro_y(i2c_bus_handle_t* bus, i2c_addr_t dev, double* data);
i2c_error_t icm_get_gyro_z(i2c_bus_handle_t* bus, i2c_addr_t dev, double* data);

i2c_error_t icm_get_gyro_x_f(i2c_bus_handle_t* bus, i2c_addr_t dev, float* data);
i2c_error_t icm_get_gyro_y_f(i2c_bus_handle_t* bus, i2c_addr_t dev, float* data);
i2c_error_t icm_get_gyro_z_f(i2c_bus_handle_t* bus, i2c_addr_t dev, float* data);

i2c_error_t icm_get_accel_x(i2c_bus_handle_t* bus, i2c_addr_t dev, double* data);
i2c_error_t icm_get_accel_y(i2c_bus_handle_t* bus, i2c_addr_t dev, double* data);
i2c_error_t icm_get_accel_z(i2c_bus_handle_t* bus, i2c_addr_t dev, double* data);

i2c_error_t icm_get_accel_x_f(i2c_bus_handle_t* bus, i2c_addr_t dev, float* data);
i2c_error_t icm_get_accel_y_f(i2c_bus_handle_t* bus, i2c_addr_t dev, float* data);
i2c_error_t icm_get_accel_z_f(i2c_bus_handle_t* bus, i2c_addr_t dev, float* data);

/**
 * Gets the tempurature in degrees celsius.
 */
i2c_error_t icm_get_temp(i2c_bus_handle_t* bus, i2c_addr_t dev, double* data);

/**
 * Gets the tempurature in degrees celsius.
 */
i2c_error_t icm_get_temp_f(i2c_bus_handle_t* bus, i2c_addr_t dev, float* data);

#endif  /* _ICM_20948 */
