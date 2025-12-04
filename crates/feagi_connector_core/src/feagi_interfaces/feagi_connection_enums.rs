
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum FeagiInterfaceStatus {
    #[default]
    Unconnected,
    Connecting,
    Retrying,
    Connected(DeviceRegistrationStatus),
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum DeviceRegistrationStatus {
    #[default]
    Unknown,
    DevicesNotMatched,
    DevicesMatched
}