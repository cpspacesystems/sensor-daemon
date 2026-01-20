#include <stdarg.h>
#include <stdio.h>
#include <fcntl.h>
#include <unistd.h>
#include <sys/ioctl.h>
#include <linux/i2c-dev.h>
#include "driver_bmp390_interface.h"


static int global_i2c_fd = -1;  //file Descriptor


uint8_t bmp390_interface_spi_init(void) { return 0; }
uint8_t bmp390_interface_spi_deinit(void) { return 0; }
uint8_t bmp390_interface_spi_read(uint8_t reg, uint8_t *buf, uint16_t len) { return 0; }
uint8_t bmp390_interface_spi_write(uint8_t reg, uint8_t *buf, uint16_t len) { return 0; }

uint8_t bmp390_interface_iic_init(void) {

    global_i2c_fd = open("/dev/i2c-1", O_RDWR);

    if (global_i2c_fd == -1) {
        printf("bmp390_interface_iic_init: open failed\n");
        return 1;
    }
    return 0;

}

uint8_t bmp390_interface_iic_deinit(void) {
    close(global_i2c_fd);
    return 0;
}

uint8_t bmp390_interface_iic_write(uint8_t addr, uint8_t reg, uint8_t *buf, uint16_t len) {
    //execute ioctl, and check if failed,
    //(file, cmd, arg). arg==destination, I2CSlave==action.
    if (ioctl(global_i2c_fd, I2C_SLAVE, addr) < 0) {
        printf("bmp390_interface_iic_write: ioctl failed\n");
        return 1;
    }

    uint8_t write_buf[len + 1];
    write_buf[0] = reg;

    memcpy(write_buf + 1, buf, len);

    if (write(global_i2c_fd, write_buf, len + 1) != len + 1) {
        printf("bmp390_interface_iic_write: write failed\n");
        return 1;
    }

    return 0;

}

uint8_t bmp390_interface_iic_read(uint8_t addr, uint8_t reg, uint8_t *buf, uint16_t len) {

     if (ioctl(global_i2c_fd, I2C_SLAVE, addr) < 0) {
        printf("bmp390_interface_iic_read: ioctl failed\n");
        return 1;
     }
    // reg -> address of register address we want to read from and register addresses are 1 byte long.
    if (write(global_i2c_fd, &reg, 1) < 0) {
        printf("bmp390_interface_iic_read: write failed\n");
        return 1;
    }

    if (read(global_i2c_fd, buf, len) != len) {
        printf("bmp390_interface_iic_read: read failed\n");
        return 1;
    }
    return 0;
}


void bmp390_interface_delay_ms(uint32_t ms) {
    usleep(ms * 1000);
}


void bmp390_interface_debug_print(const char *const fmt, ...) {
    va_list args;
    va_start(args, fmt);
    vprintf(fmt, args);
    va_end(args);
}


