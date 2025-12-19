#[derive(Debug, Clone, Copy, PartialEq, Default, Hash)]
pub enum FeagiInterfaceStatus {
    #[default]
    Unconnected,
    Connecting,
    Retrying,
    Connected(DeviceRegistrationStatus),
}

#[derive(Debug, Clone, Copy, PartialEq, Default, Hash)]
pub enum DeviceRegistrationStatus {
    #[default]
    Unknown,
    DevicesNotMatched,
    DevicesMatched,
}
