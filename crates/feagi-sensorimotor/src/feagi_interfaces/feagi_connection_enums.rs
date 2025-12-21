#[derive(Debug, Clone, Copy, PartialEq, Default, Hash)]
#[allow(dead_code)]
pub enum FeagiInterfaceStatus {
    #[default]
    Unconnected,
    Connecting,
    Retrying,
    Connected(DeviceRegistrationStatus),
}

#[derive(Debug, Clone, Copy, PartialEq, Default, Hash)]
#[allow(dead_code)]
pub enum DeviceRegistrationStatus {
    #[default]
    Unknown,
    DevicesNotMatched,
    DevicesMatched,
}
