//! The original ESP32. effectively ~320 kbs for us to play with. May have PSRAM. cannot do SIMD

// NOTE: for now dont use PSRAM. This may be the best choice for performance and if we need to leave
// it to the camera for sensiomotor stuff. We should only consider the PSRAM for deployments where we
// need a lot of neurons and dont care about speed, nor have a device that is using a lot of RAM as well

pub mod burst_engine;