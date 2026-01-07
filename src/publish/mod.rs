mod shm;

use crate::frame::SensorFrame;
use std::io;
use zenoh::Wait;

#[cfg(feature = "publish_shm")]
use crate::publish::shm::SharedMemory;

/// The name used for the shared memory allocation if the shared memory backend is selected.
#[cfg(feature = "publish_shm")]
const SHM_NAME: &'static str = "/sensord";

#[cfg(any(feature = "publish_zenoh", feature = "publish_zenoh_shm"))]
const ZENOH_PUB_KEY: &'static str = "cpss/sensor-data";

/// Backend agnostic [`SensorFrame`] publisher, for publishing [`SensorFrame`]s to other processes.
///
/// [`SensorFrame`]: SensorFrame
#[cfg(feature = "publish_shm")]
pub struct Publisher {
    shm: SharedMemory<SensorFrame>,
}

#[cfg(any(feature = "publish_zenoh", feature = "publish_zenoh_shm"))]
pub struct Publisher<'a> {
    _session: zenoh::Session,
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

#[cfg(any(feature = "publish_zenoh", feature = "publish_zenoh_shm"))]
impl<'a> Publisher<'a> {
    pub fn new() -> io::Result<Publisher<'a>> {
        let config = zenoh::Config::default();
        let session = zenoh::open(config)
            .wait()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let publisher = session
            .declare_publisher(ZENOH_PUB_KEY)
            .wait()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        Ok(Publisher {
            _session: session,
            publisher,
        })
    }

    #[cfg(feature = "publish_zenoh")]
    pub fn publish_frame(&mut self, frame: SensorFrame) -> io::Result<()> {
        self.publisher
            .put(frame.to_bytes())
            .wait()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        Ok(())
    }

    #[cfg(feature = "publish_zenoh_shm")]
    pub fn publish_frame(&mut self, frame: SensorFrame) -> io::Result<()> {
        let provider =
            zenoh::shm::ShmProviderBuilder::default_backend(zenoh::shm::MemoryLayout::for_type::<
                SensorFrame,
            >())
            .wait()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let shared_buf = provider
            .alloc(zenoh::shm::TypedLayout::<SensorFrame>::new())
            .wait()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
            .initialize(frame);

        self.publisher
            .put(shared_buf)
            .wait()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        Ok(())
    }
}
