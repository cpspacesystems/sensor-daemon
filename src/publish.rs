//! Module for publishing sensor frames. Which essentially comes down to being a wrapper for
//! [`tism`]'s lazy API.
//!
//! [`tism`]: tism

use crate::frame::SensorFrame;
use std::io;

/// The name of the shared memory allocation used by sensor daemon.
pub const SHM_NAME: &'static str = "sensord";

/// Owning type of the resources required to publish data to other processes.
pub struct Publisher {
    shm: tism::lazy::LazyOwnedSharedMemory<SensorFrame, &'static str>,
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
        self.shm.write(frame)?;
        Ok(())
    }
}
