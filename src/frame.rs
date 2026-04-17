//! Module for frames of sensor info, this module is common between the publishing and all consuming
//! parts of sensor daemon. This module hold the data definitions and a few utility functions which
//! are either necessary or helpful in handling sensor information.

use crate::drivers::{bmp390::Bmp390Frame, icm_20948::Icm20948Frame};
use libc::{self, timeval};
use std::{
    io,
    mem::{self, MaybeUninit},
    ptr,
    time::{Duration, SystemTime},
};

/// [`SensorFrameData`] with a timestamp. This is the payload for publishing.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct SensorFrame {
    /// `timestamp` is a linux `struct timeval` to allow for easy inter-op with C code.
    timestamp: libc::timeval,

    /// Data for the frame.
    data: SensorFrameData,
}

impl SensorFrame {
    /// Get the timestamp, as a [`SystemTime`], with which the [`SensorFrame`] was stamped.
    ///
    /// [`SystemTime`]: SystemTime
    /// [`SensorFrame`]: SensorFrame
    pub fn time(&self) -> SystemTime {
        // This function is kinda funny, since Rust just uses a structure identical to `timeval`
        // anyways (at least on unix-like systems), so this _could_ basically be a transmute but
        // doing this way is safer and checks everything into the type system a bit better.
        SystemTime::UNIX_EPOCH
            + Duration::from_secs(self.timestamp.tv_sec as _)
            + Duration::from_micros(self.timestamp.tv_usec as _)
    }

    /// Create an array of `u8` bytes which are, bitwise, exactly the same as the given
    /// [`SensorFrame`].
    ///
    /// [`SensorFrame`]: SensorFrame
    pub fn to_bytes(self) -> [u8; size_of::<Self>()] {
        unsafe {
            let self_ptr: *const Self = ptr::from_ref(&self);
            let arr_ptr: *const [u8; size_of::<Self>()] = mem::transmute(self_ptr);
            *arr_ptr
        }
    }

    /// Creates a [`SensorFrame`] which is identical, bitwise, to the given byte array.
    ///
    /// [`SensorFrame`]: SensorFrame
    pub fn from_bytes(bytes: [u8; size_of::<Self>()]) -> Self {
        unsafe {
            let self_ref: &Self = mem::transmute(bytes.as_ptr());
            self_ref.to_owned()
        }
    }
}

/// The data we are concerned with transmitting to other processes.
#[repr(C)]
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct SensorFrameData {
    pub gyro_data: Icm20948Frame,
    pub altimeter_data: Bmp390Frame,
}

/// Timestamp the given [`SensorFrameData`], returning a [`SensorFrame`] stamped with the current
/// system time.
///
/// [`SensorFrame`]: SensorFrame
/// [`SensorFrameData`]: SensorFrameData
pub fn timestamp_frame(data: SensorFrameData) -> io::Result<SensorFrame> {
    let mut timestamp: MaybeUninit<timeval> = MaybeUninit::uninit();

    unsafe {
        if libc::gettimeofday(timestamp.as_mut_ptr(), ptr::null_mut()) != 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(SensorFrame {
            timestamp: timestamp.assume_init(),
            data,
        })
    }
}
