use crate::next::FeagiNetworkError;
use crate::next::traits_and_enums::server::server_enums::FeagiServerBindState;

/// Base trait for all FEAGI server types.
///
/// Provides common lifecycle management for server sockets, including
/// binding to an address and tracking the current connection state.
///
/// All specialized server traits ([`super::FeagiServerPublisher`],
/// [`super::FeagiServerPuller`], [`super::FeagiServerRouter`]) extend this trait.
pub trait FeagiServer {
    /// Binds the server socket to the configured address and starts listening.
    ///
    /// # Errors
    /// Returns [`FeagiNetworkError::CannotBind`] if the socket cannot bind to the address.
    fn start(&mut self) -> Result<(), FeagiNetworkError>;

    /// Unbinds the server socket from the address and stops listening.
    ///
    /// # Errors
    /// Returns [`FeagiNetworkError::CannotUnbind`] if the socket cannot be unbound.
    fn stop(&mut self) -> Result<(), FeagiNetworkError>;

    /// Returns the current bind state of the server.
    fn get_current_state(&self) -> FeagiServerBindState;

    /// Sets a function that gets immediately following the bind state changing
    fn set_callback_on_state_change<F>(&self, callback: F) where
        F: Fn(FeagiServerBindStateChange) + Send + Sync + 'static;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FeagiServerBindStateChange {
    previous: FeagiServerBindState,
    now: FeagiServerBindState,
}
