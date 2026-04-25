use gpsd_client::{GPS, GPSData, GPSError};

#[derive(Debug)]
pub struct NeoM9 {
    gpsd: GPS,
}

/// A single frame of data from a Neo M9.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NeoM9Frame {
    /// Speed over the ground in meters per second.
    speed: f32,
    /// Heading over the ground in degrees from true north.
    heading: f32,
    /// Latitude in degrees.
    latitude: f64,
    /// Longitude in degrees.
    longitude: f64,
    /// Estimated speed error in meters per second with 95% confidence.
    estimated_speed_error: f32,
    /// Estimated longitude error in degrees with 95% confidence.
    estimated_longitude_error: f32,
    /// Estimated latitude error in degrees with 95% confidence.
    estimated_latitude_error: f32,
    /// Number of connected satalites.
    satalites: u8,
    /// Number of satalites with valid data.
    valid_satalites: u8,
}

impl NeoM9 {
    /// Open the GPS daemon for use with a Neo M9 GPS board.
    pub fn open() -> Result<NeoM9, GPSError> {
        Ok(NeoM9 {
            gpsd: GPS::connect()?,
        })
    }

    /// Get a new frame of GPS data from the [`NeoM9`].
    ///
    /// [`NeoM9`]: NeoM9
    pub fn read(&mut self) -> Result<NeoM9Frame, GPSError> {
        let data = self.gpsd.current_data()?;
        Ok(data.into())
    }
}

impl From<GPSData> for NeoM9Frame {
    fn from(v: GPSData) -> Self {
        NeoM9Frame {
            speed: v.speed,
            heading: v.track,
            satalites: v.sats,
            valid_satalites: v.sats_valid,
            latitude: v.lat,
            longitude: v.lon,
            estimated_speed_error: v.eps,
            estimated_longitude_error: v.epx,
            estimated_latitude_error: v.epy,
        }
    }
}
