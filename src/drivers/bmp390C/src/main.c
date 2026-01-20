#include <stdio.h>
#include "driver_bmp390.h"
#include "driver_bmp390_interface.h"


bmp390_handle_t g_bmp390_handle;

int main(void) {

    uint8_t chip_id;
    float temperature, pressure;
    uint32_t raw_temp, raw_press;

    printf("Starting BMP390 Driver Test\n");

    DRIVER_BMP390_LINK_INIT(&g_bmp390_handle, bmp390_handle_t);
    DRIVER_BMP390_LINK_IIC_INIT(&g_bmp390_handle, bmp390_interface_iic_init);
    DRIVER_BMP390_LINK_IIC_DEINIT(&g_bmp390_handle, bmp390_interface_iic_deinit);
    DRIVER_BMP390_LINK_IIC_READ(&g_bmp390_handle, bmp390_interface_iic_read);
    DRIVER_BMP390_LINK_IIC_WRITE(&g_bmp390_handle, bmp390_interface_iic_write);
    DRIVER_BMP390_LINK_DELAY_MS(&g_bmp390_handle, bmp390_interface_delay_ms);
    DRIVER_BMP390_LINK_DEBUG_PRINT(&g_bmp390_handle, bmp390_interface_debug_print);



    DRIVER_BMP390_LINK_SPI_INIT(&g_bmp390_handle, bmp390_interface_spi_init);
    DRIVER_BMP390_LINK_SPI_DEINIT(&g_bmp390_handle, bmp390_interface_spi_deinit);
    DRIVER_BMP390_LINK_SPI_READ(&g_bmp390_handle, bmp390_interface_spi_read);
    DRIVER_BMP390_LINK_SPI_WRITE(&g_bmp390_handle, bmp390_interface_spi_write);
    //I2C (Address Low = 0x76, High = 0x77)

    // If SDO is to GND, use ADO_LOW. If to VCC, use ADO_HIGH.
    bmp390_set_interface(&g_bmp390_handle, BMP390_INTERFACE_IIC);
    bmp390_set_addr_pin(&g_bmp390_handle, BMP390_ADDRESS_ADO_HIGH);

    uint8_t res = bmp390_init(&g_bmp390_handle);
    if (res != 0) {
        printf("Init failed with error code: %d\n", res);
        return 1;
    }
    printf("Init successful!\n");

    //Read cheap id
    res = bmp390_get_revision_id(&g_bmp390_handle, &chip_id);
    if (res != 0) {
        printf("Failed to read Chip ID.\n");
    } else {
        printf("Chip ID: 0x%02X\n", chip_id);
    }


    bmp390_set_odr(&g_bmp390_handle, BMP390_ODR_200_HZ);
    bmp390_set_filter_coefficient(&g_bmp390_handle, BMP390_FILTER_COEFFICIENT_0); //Disable IIR filtering
    // 6. Configure for Reading (Normal Mode, Enable Temp/Press)
    bmp390_set_pressure(&g_bmp390_handle, BMP390_BOOL_TRUE);
    bmp390_set_temperature(&g_bmp390_handle, BMP390_BOOL_TRUE);
    bmp390_set_mode(&g_bmp390_handle, BMP390_MODE_NORMAL_MODE);


    //Read Data
    int i = 0;
    while (1) {
        bmp390_interface_delay_ms(500);

        res = bmp390_read_temperature_pressure(&g_bmp390_handle,
                                               &raw_temp, &temperature,
                                               &raw_press, &pressure);

        if (res == 0) {
            printf("[%d] Temp: %.4f C | Press: %.4f Pa\n", i, temperature, pressure);
        } else {
            printf("Read failed.\n");
        }
    }

    bmp390_deinit(&g_bmp390_handle);
    return 0;
}