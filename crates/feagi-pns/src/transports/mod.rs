//! Transport implementations
//!
//! Each transport module implements either BlockingTransport or NonBlockingTransport
//! (or both for dual-mode transports like ZMQ).
//!
//! Available transports:
//! - zmq: ZeroMQ (current: blocking, future: nonblocking)
//! - shm: Shared Memory (blocking only)
//! - udp: User Datagram Protocol (nonblocking only, future)
//! - websocket: WebSocket (nonblocking only, future)
//! - rtos: Embedded/RTOS (special no_std, future)

pub mod zmq;

// Future transports (placeholder modules will be added as needed):
// pub mod udp;
// pub mod shm;
// pub mod websocket;
// pub mod rtos;

