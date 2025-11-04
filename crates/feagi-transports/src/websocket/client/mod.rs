//! WebSocket client implementations
//!
//! Provides client-side WebSocket transports for FEAGI agents:
//! - Subscriber (SUB): Subscribe to broadcasts
//! - Push: Send messages to server
//! - Dealer: Request/reply client

pub mod sub;
pub mod push;
pub mod dealer;

pub use sub::WsSub;
pub use push::WsPush;
pub use dealer::WsDealer;

