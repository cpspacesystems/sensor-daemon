mod shm;

use crate::frame::SensorFrame;
use std::io;

#[cfg(feature = "publish_shm")]
use crate::publish::shm::SharedMemory;

/// The name used for the shared memory allocation if the shared memory backend is selected.
#[cfg(feature = "publish_shm")]
const SHM_NAME: &'static str = "/sensord";

#[cfg(feature = "publish_zenoh")]
const ZENOH_PUB_KEY: &'static str = "cpss/sensor-data";

/// Backend agnostic [`SensorFrame`] publisher, for publishing [`SensorFrame`]s to other processes.
///
/// [`SensorFrame`]: SensorFrame
#[cfg(feature = "publish_shm")]
pub struct Publisher {
    shm: SharedMemory<SensorFrame>,
}

#[cfg(feature = "publish_zenoh")]
pub struct Publisher<'a> {
    session: zenoh::Session,
    publisher: zenoh::pubsub::Publisher<'a>,
}

#[cfg(feature = "publish_shm")]
impl Publisher {
    /// Create a new [`Publisher`]. The inner workings of this function, as well as what recourses
    /// it may use, are entirely dependant on what features are enabled, as that sets the backend
    /// for the [`Publisher`] as a whole.
    ///
    /// [`Publisher`]: Publisher
    pub fn new() -> io::Result<Publisher> {
        let shm = SharedMemory::create(SHM_NAME)?;
        Ok(Publisher { shm })
    }

    pub fn publish_frame(&mut self, frame: SensorFrame) -> io::Result<()> {
        self.shm.set(frame)?;
        Ok(())
    }
}

#[cfg(feature = "publish_zenoh")]
impl<'a> Publisher<'a> {
    pub fn new() -> io::Result<Publisher<'a>> {
        let config = zenoh::Config::default();
        let session = zenoh::Wait::wait(zenoh::open(config))
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let publisher = zenoh::Wait::wait(session.declare_publisher(ZENOH_PUB_KEY))
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        Ok(Publisher { session, publisher })
    }

    pub fn publish_frame(&mut self, frame: SensorFrame) -> io::Result<()> {
        zenoh::Wait::wait(self.publisher.put(frame.to_bytes()))
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        Ok(())
    }
}
