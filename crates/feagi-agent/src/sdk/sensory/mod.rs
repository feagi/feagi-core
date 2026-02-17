//! Sensory encoding (video, text, etc.). SDK surface for controllers.

#[cfg(feature = "sdk-text")]
pub mod text;
#[cfg(feature = "sdk-video")]
pub mod traits;
#[cfg(feature = "sdk-video")]
pub mod video;
