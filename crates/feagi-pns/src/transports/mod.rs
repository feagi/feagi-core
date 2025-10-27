//! Transport implementations
//!
//! Each transport module implements either BlockingTransport or NonBlockingTransport
//! (or both for dual-mode transports like ZMQ).
//!
//! Available transports:
//! - zmq: ZeroMQ (blocking, TCP-based, reliable)
//! - udp: User Datagram Protocol (nonblocking, best-effort, high-throughput)
//! - shm: Shared Memory (blocking only, future)
//! - websocket: WebSocket (nonblocking only, future)
//! - rtos: Embedded/RTOS (special no_std, future)

pub mod zmq;
pub mod udp;

// Future transports (placeholder modules will be added as needed):
// pub mod shm;
// pub mod websocket;
// pub mod rtos;

