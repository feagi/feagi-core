# `feagi-agent::clients` (Session Orchestration)

This module contains **client-side orchestration** building blocks for FEAGI agents.

## Goals

- **Runtime-agnostic**: core logic does not depend on Tokio, threads, or sleeps (WASM / Embassy / RTOS friendly).
- **Deterministic**: the orchestration layer is a pure state machine driven by explicit inputs.
- **Transport-agnostic**: **ZMQ and WebSocket are first-class** via `TransportProtocolEndpoint`.

## Key Types

- **`SessionStateMachine`**: pure state machine that orchestrates registration and data-channel bring-up.
  - Inputs: `SessionEvent` values (observations from polling network clients).
  - Outputs: `SessionAction` values (instructions for the driver to execute I/O).
  - Timing: uses driver-provided monotonic time (`NowMs`) and `SessionTimingConfig`.

- **`CommandControlAgent`**: poll-based requester for registration/heartbeat/deregistration.

- **Tokio adapter (feature-gated)**: `clients::async_helpers::tokio_generic_implementations`
  - `TokioEmbodimentAgent` drives `SessionStateMachine` and executes actions using `feagi-io` clients.
  - This is an adapter layer; it is intentionally runtime-specific.

## Transport support (ZMQ + WebSocket)

`TransportProtocolEndpoint` provides **fallible** factory helpers:

- `try_create_boxed_client_requester_properties()`
- `try_create_boxed_client_pusher_properties()`
- `try_create_boxed_client_subscriber_properties()`

These are preferred over the older `create_*` variants because they **do not panic** and allow you to propagate configuration/endpoint errors.

## Minimal driver sketch (runtime-agnostic)

The session state machine does not perform I/O. A driver should:

1. Poll network clients to collect `SessionEvent`s.
2. Call `SessionStateMachine::step(now_ms, &events)`.
3. Execute returned `SessionAction`s (connect, send registration, connect data channels, send heartbeat).

That driver can be implemented for:

- Tokio (desktop/server)
- WASM event loop
- Embassy/RTOS cooperative scheduler

