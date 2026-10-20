//! The base implementations for the FEAGI library of channels. Using either Thingbuf channels
//! as a basis, or in some cases embassy. United by common traits

pub mod static_channels;
pub mod alloc_channels;
pub mod basis;