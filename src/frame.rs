//! Module for frames of sensor info, this module is common between the publishing and all consuming
//! parts of sensor daemon. This module hold the data definitions and a few utility functions which
//! are either necessary or helpful in handling sensor information.

use crate::drivers::{bmp390::Bmp390Frame, icm_20948::Icm20948Frame, neo_m9::NeoM9Frame};

/// The data we are concerned with transmitting to other processes.
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct SensorFrame {
    pub gyro_data: Icm20948Frame,
    pub altimeter_data: Bmp390Frame,
    pub gps_data: NeoM9Frame,
}
