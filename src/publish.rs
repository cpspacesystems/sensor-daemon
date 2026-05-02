//! Module for publishing sensor frames. Which essentially comes down to being a wrapper for
//! [`tism`]'s lazy API.
//!
//! [`tism`]: tism

use crate::{fb, frame::SensorFrame};
use std::io;

/// The name of the shared memory allocation used by sensor daemon.
pub const SHM_NAME: &'static str = "sensord";

/// Size of the flatbuffer.
pub const FRAME_SIZE: usize = 78;

/// Owning type of the resources required to publish data to other processes.
pub struct Publisher {
    shm: tism::lazy::LazyOwnedSharedMemory<[u8; FRAME_SIZE], &'static str>,
}

impl Publisher {
    /// Create a new [`Publisher`]. This will _not_ make data available to other processes, to do
    /// that you must write to the [`Publisher`] with [`Publisher::publish_frame`].
    ///
    /// [`Publisher`]: Publisher
    /// [`Publisher::publish_frame`]: Publisher::publish_frame
    pub fn new() -> io::Result<Publisher> {
        let shm = tism::lazy::create(SHM_NAME);
        Ok(Publisher { shm })
    }

    /// Publish a [`SensorFrame`], making it available to other processes.
    ///
    /// [`SensorFrame`]: SensorFrame
    pub fn publish_frame(&mut self, frame: SensorFrame) -> io::Result<()> {
        let fb = fb::create_fb_frame(frame);

        if fb.len() != FRAME_SIZE {
            eprintln!(
                "Framebuffer length and `FRAME_SIZE` don't match! ({} and {})",
                fb.len(),
                FRAME_SIZE
            );

            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Flatbuffer length and `FRAME_SIZE` don't match! ({} and {})",
                    fb.len(),
                    FRAME_SIZE
                ),
            ));
        }

        let arr = match fb.as_array::<FRAME_SIZE>() {
            Some(arr) => *arr,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Could not make fixed size array from flatbuffer slice",
                ));
            }
        };

        self.shm.write(arr)?;
        Ok(())
    }
}
