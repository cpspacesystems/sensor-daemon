//! The TOM 2.0 sensor daemon, as exposed for consumption in Rust projects. This library crate is
//! not intended for publishing/writing sensor data, and as such will not export the capability to
//! do so.

pub mod consumption;
pub mod frame;
mod publish;
