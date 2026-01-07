//! Intended for processes wishing to consume the output of sensor daemon, this module provides a
//! backend agnostic interface for retrieving [`SensorFrame`]s published by sensor daemon.
//!
//! [`SensorFrame`]: crate::frame::SensorFrame

use crate::{frame::SensorFrame, publish};
use std::io;

#[cfg(any(feature = "publish_zenoh", feature = "publish_zenoh_shm"))]
use zenoh::Wait;

/// Create a new [`Consumer`]. The exact struct used here depends on the selected feature set at
/// build time and should not be the concern of a program using this module.
///
/// [`Consumer`]: Consumer
pub fn make_consumer() -> io::Result<impl Consumer> {
    #[cfg(any(feature = "publish_zenoh", feature = "publish_zenoh_shm"))]
    return ZenohConsumer::new().map_err(|e| io::Error::new(io::ErrorKind::Other, e));

    #[cfg(feature = "publish_shm")]
    return ShmConsumer::new();

    #[cfg(not(any(
        feature = "publish_shm",
        feature = "publish_zenoh",
        feature = "publish_zenoh_shm"
    )))]
    return Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "No backend selected",
    ));
}

pub trait Consumer {
    /// Get the current/latest [`SensorFrame`] which has been published by sensor daemon.
    ///
    /// [`SensorFrame`]: SensorFrame
    fn get_frame(&mut self) -> io::Result<SensorFrame>;
}

/// A [`Consumer`] using a shared memory backend.
///
/// [`Consumer`]: Consumer
#[cfg(feature = "publish_shm")]
struct ShmConsumer {
    shm: publish::shm::SharedMemory<SensorFrame>,
}

#[cfg(feature = "publish_shm")]
impl ShmConsumer {
    /// Create a new [`ShmConsumer`] using the same allocation as would be used by the sensor
    /// daemon.
    ///
    /// [`ShmConsumer`]: ShmConsumer
    fn new() -> io::Result<ShmConsumer> {
        publish::shm::SharedMemory::open(publish::SHM_NAME).map(|shm| ShmConsumer { shm })
    }
}

#[cfg(feature = "publish_shm")]
impl Consumer for ShmConsumer {
    fn get_frame(&mut self) -> io::Result<SensorFrame> {
        self.shm.get()
    }
}

/// A [`Consumer`] for interfacing with sensor daemon when any zenoh backend is selected.
///
/// [`Consumer`]: Consumer
#[cfg(any(feature = "publish_zenoh", feature = "publish_zenoh_shm"))]
struct ZenohConsumer<'q> {
    _session: zenoh::Session,
    querier: zenoh::query::Querier<'q>,
}

#[cfg(any(feature = "publish_zenoh", feature = "publish_zenoh_shm"))]
impl<'q> ZenohConsumer<'q> {
    /// Create a new [`ZenohConsumer`].
    ///
    /// [`ZenohConsumer`]: ZenohConsumer
    fn new() -> zenoh::Result<Self> {
        let config = zenoh::Config::default();
        let session = zenoh::open(config).wait()?;
        let querier = session.declare_querier(publish::ZENOH_PUB_KEY).wait()?;

        Ok(Self {
            _session: session,
            querier,
        })
    }
}

#[cfg(any(feature = "publish_zenoh", feature = "publish_zenoh_shm"))]
impl<'q> Consumer for ZenohConsumer<'q> {
    fn get_frame(&mut self) -> io::Result<SensorFrame> {
        let reciever = self
            .querier
            .get()
            .wait()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let recv = reciever
            .recv()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let sample = recv
            .result()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.clone()))?;

        let bytes = sample.payload().to_bytes();
        let frame = bytes.as_array().map(|arr| SensorFrame::from_bytes(*arr));

        frame.ok_or(io::Error::new(
            io::ErrorKind::InvalidData,
            "Recieved bad number of bytes",
        ))
    }
}
