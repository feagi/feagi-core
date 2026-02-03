//! Client-side networking traits for FEAGI.
//!
//! This module defines the core abstractions for client-side network communication,
//! supporting multiple messaging patterns:
//!
//! - **Pusher** ([`FeagiClientPusher`]): Push data to a server (fire-and-forget)
//! - **Requester** ([`FeagiClientRequester`]): Request-response pattern
//! - **Subscriber** ([`FeagiClientSubscriber`]): Receive broadcast data from a publisher

mod feagi_client;
mod feagi_client_pusher;
mod feagi_client_requester;
mod feagi_client_subscriber;

pub use feagi_client::FeagiClient;
pub use feagi_client_pusher::FeagiClientPusher;
pub use feagi_client_requester::FeagiClientRequester;
pub use feagi_client_subscriber::FeagiClientSubscriber;
