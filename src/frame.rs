use libc::{self, timeval};
use std::{
    io,
    mem::{self, MaybeUninit},
    ptr, slice,
};

/// [`SensorFrameData`] with a timestemp. This is the payload for publishing.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct SensorFrame {
    /// `timestamp` is a linux `struct timeval` to allow for easy inter-op with C code.
    timestamp: libc::timeval,

    /// Data for the frame.
    data: SensorFrameData,
}

impl SensorFrame {
    /// Returns a slice of bytes which points to the same memory as `self`.
    pub fn to_bytes(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(mem::transmute(ptr::from_ref(self)), size_of::<Self>()) }
    }

    pub fn from_bytes(bytes: &[u8]) -> &Self {
        unsafe { mem::transmute(bytes.as_ptr().as_ref()) }
    }
}

/// The data we are concerned with transmitting to other processes.
#[repr(C)]
#[derive(Debug, PartialEq, Copy, Clone, Default)]
pub struct SensorFrameData {
    pub accel_x: f64,
    pub accel_y: f64,
    pub accel_z: f64,
    pub gyro_x: f64,
    pub gyro_y: f64,
    pub gyro_z: f64,
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
