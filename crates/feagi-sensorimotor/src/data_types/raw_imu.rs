//! Raw IMU (Inertial Measurement Unit) composite sensor type.
//!
//! Holds three independent 3-axis vectors that together describe a raw IMU
//! reading: accelerometer, gyroscope, and magnetometer. Each sub-component is a
//! signed 3-D percentage in the range [-1.0, 1.0] per axis after the controller
//! has normalized the raw physical units against its configured ranges.
//!
//! This composite is the wrapped-data counterpart of
//! [`SensoryCorticalUnit::RawIMU`], which is registered as a SINGLE cortical_area
//! unit owning THREE sub-cortical_area-areas (one per sub-component). The matching
//! [`RawIMUNeuronVoxelXYZPEncoder`] is responsible for spreading this composite
//! across those three sub-areas at neuron-encoding time.
//!
//! Sub-component ordering for all ordered accessors (and the encoder's
//! `cortical_write_targets` array) is fixed and authoritative:
//!   index 0 -> accelerometer
//!   index 1 -> gyroscope
//!   index 2 -> magnetometer
//!
//! Quaternion-based orientation is intentionally NOT part of this composite;
//! see `SensoryCorticalUnit::SmartIMU` (a 4-D signed percentage) for that.

use super::{SignedPercentage, SignedPercentage3D};
use feagi_structures::FeagiDataError;

/// Number of sub-cortical_area-areas backing a Raw IMU reading: accel, gyro, mag.
pub const RAW_IMU_SUBUNIT_COUNT: usize = 3;

/// Stable index of the accelerometer sub-area within Raw IMU ordered slots.
pub const RAW_IMU_INDEX_ACCELEROMETER: usize = 0;
/// Stable index of the gyroscope sub-area within Raw IMU ordered slots.
pub const RAW_IMU_INDEX_GYROSCOPE: usize = 1;
/// Stable index of the magnetometer sub-area within Raw IMU ordered slots.
pub const RAW_IMU_INDEX_MAGNETOMETER: usize = 2;

/// Composite Raw IMU reading: three independent 3-axis signed percentages.
///
/// Each sub-component carries a `SignedPercentage3D` (axes a/b/c, each in
/// `[-1.0, 1.0]`). The cortical_area-area dimension contract for each sub-area is
/// `[3, 1, z]` (3 channels = 3 axes, height 1, configurable Z resolution).
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RawIMU {
    /// Linear acceleration along x/y/z axes.
    accelerometer: SignedPercentage3D,
    /// Angular velocity around x/y/z axes (NOT orientation; that is `SmartIMU`).
    gyroscope: SignedPercentage3D,
    /// Magnetic-field reading along x/y/z axes.
    magnetometer: SignedPercentage3D,
}

impl RawIMU {
    /// Construct a new Raw IMU reading from its three sub-components.
    pub fn new(
        accelerometer: SignedPercentage3D,
        gyroscope: SignedPercentage3D,
        magnetometer: SignedPercentage3D,
    ) -> Self {
        Self {
            accelerometer,
            gyroscope,
            magnetometer,
        }
    }

    /// Construct an all-zero reading (used as the cache's initial value).
    pub fn new_zero() -> Self {
        Self {
            accelerometer: SignedPercentage3D::new_identical_percentages(
                SignedPercentage::new_from_m1_1_unchecked(0.0),
            ),
            gyroscope: SignedPercentage3D::new_identical_percentages(
                SignedPercentage::new_from_m1_1_unchecked(0.0),
            ),
            magnetometer: SignedPercentage3D::new_identical_percentages(
                SignedPercentage::new_from_m1_1_unchecked(0.0),
            ),
        }
    }

    /// Construct from raw f32 axis triples (accel, gyro, mag) in `[-1.0, 1.0]`.
    pub fn try_from_axis_triples(
        accelerometer_xyz: (f32, f32, f32),
        gyroscope_xyz: (f32, f32, f32),
        magnetometer_xyz: (f32, f32, f32),
    ) -> Result<Self, FeagiDataError> {
        let accelerometer: SignedPercentage3D = accelerometer_xyz.try_into()?;
        let gyroscope: SignedPercentage3D = gyroscope_xyz.try_into()?;
        let magnetometer: SignedPercentage3D = magnetometer_xyz.try_into()?;
        Ok(Self::new(accelerometer, gyroscope, magnetometer))
    }

    pub fn get_accelerometer(&self) -> &SignedPercentage3D {
        &self.accelerometer
    }

    pub fn get_gyroscope(&self) -> &SignedPercentage3D {
        &self.gyroscope
    }

    pub fn get_magnetometer(&self) -> &SignedPercentage3D {
        &self.magnetometer
    }

    /// Replace just the accelerometer sub-component, leaving gyro and mag intact.
    pub fn set_accelerometer(&mut self, value: SignedPercentage3D) {
        self.accelerometer = value;
    }

    /// Replace just the gyroscope sub-component, leaving accel and mag intact.
    pub fn set_gyroscope(&mut self, value: SignedPercentage3D) {
        self.gyroscope = value;
    }

    /// Replace just the magnetometer sub-component, leaving accel and gyro intact.
    pub fn set_magnetometer(&mut self, value: SignedPercentage3D) {
        self.magnetometer = value;
    }

    /// Ordered references in the canonical sub-area order: accel, gyro, mag.
    ///
    /// The order here MUST match the order used to build the encoder's
    /// `cortical_write_targets` array, otherwise sub-areas will receive each
    /// other's data.
    pub fn get_ordered_sub_components(&self) -> [&SignedPercentage3D; RAW_IMU_SUBUNIT_COUNT] {
        [&self.accelerometer, &self.gyroscope, &self.magnetometer]
    }
}

impl std::fmt::Display for RawIMU {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "RawIMU(accel={}, gyro={}, mag={})",
            self.accelerometer, self.gyroscope, self.magnetometer
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_zero_yields_zeroed_components() {
        let imu = RawIMU::new_zero();
        let zero = SignedPercentage::new_from_m1_1_unchecked(0.0);
        assert_eq!(imu.get_accelerometer().a, zero);
        assert_eq!(imu.get_accelerometer().b, zero);
        assert_eq!(imu.get_accelerometer().c, zero);
        assert_eq!(imu.get_gyroscope().a, zero);
        assert_eq!(imu.get_magnetometer().c, zero);
    }

    /// Sub-component ordering is contractual: accel(0), gyro(1), mag(2). The
    /// encoder's per-subarea cortical_area ID array depends on this order.
    #[test]
    fn ordered_sub_components_yield_accel_gyro_mag() {
        let accel: SignedPercentage3D = (0.1, 0.2, 0.3).try_into().unwrap();
        let gyro: SignedPercentage3D = (-0.4, -0.5, -0.6).try_into().unwrap();
        let mag: SignedPercentage3D = (0.7, -0.8, 0.9).try_into().unwrap();
        let imu = RawIMU::new(accel, gyro, mag);

        let ordered = imu.get_ordered_sub_components();
        assert_eq!(ordered.len(), RAW_IMU_SUBUNIT_COUNT);
        assert_eq!(ordered[RAW_IMU_INDEX_ACCELEROMETER].a.get_as_m1_1(), 0.1);
        assert_eq!(ordered[RAW_IMU_INDEX_GYROSCOPE].a.get_as_m1_1(), -0.4);
        assert_eq!(ordered[RAW_IMU_INDEX_MAGNETOMETER].c.get_as_m1_1(), 0.9);
    }

    /// Setters update only the targeted sub-component; cross-talk would yield
    /// silently corrupted readings on the brain side.
    #[test]
    fn setters_do_not_disturb_other_sub_components() {
        let mut imu = RawIMU::new_zero();
        let new_gyro: SignedPercentage3D = (0.5, 0.5, 0.5).try_into().unwrap();
        imu.set_gyroscope(new_gyro);

        let zero = SignedPercentage::new_from_m1_1_unchecked(0.0);
        assert_eq!(imu.get_accelerometer().a, zero);
        assert_eq!(imu.get_accelerometer().b, zero);
        assert_eq!(imu.get_magnetometer().c, zero);
        assert_eq!(imu.get_gyroscope().a.get_as_m1_1(), 0.5);
    }

    #[test]
    fn try_from_axis_triples_clamps_round_trip() {
        let imu = RawIMU::try_from_axis_triples((0.1, -0.2, 0.3), (0.4, 0.5, 0.6), (0.7, 0.8, 0.9))
            .expect("Valid normalized axis triples must convert");

        assert_eq!(imu.get_accelerometer().a.get_as_m1_1(), 0.1);
        assert_eq!(imu.get_gyroscope().c.get_as_m1_1(), 0.6);
        assert_eq!(imu.get_magnetometer().b.get_as_m1_1(), 0.8);
    }

    #[test]
    fn try_from_axis_triples_rejects_out_of_range() {
        let result =
            RawIMU::try_from_axis_triples((1.5, 0.0, 0.0), (0.0, 0.0, 0.0), (0.0, 0.0, 0.0));
        assert!(
            result.is_err(),
            "Values outside [-1.0, 1.0] must error to surface controller-side normalization bugs"
        );
    }
}
