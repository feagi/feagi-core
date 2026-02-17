#[cfg(not(feature = "agent-client-asynchelper-tokio"))]
fn main() {
    eprintln!(
        "This example requires feature 'agent-client-asynchelper-tokio'. Enable it when building the crate."
    );
}

#[cfg(feature = "agent-client-asynchelper-tokio")]
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use feagi_agent::clients::async_helpers::tokio_generic_implementations::{
        TokioDriverConfig, TokioEmbodimentAgent,
    };
    use feagi_agent::clients::SessionTimingConfig;
    use feagi_agent::{AgentCapabilities, AgentDescriptor, AuthToken};
    use feagi_io::protocol_implementations::websocket::websocket_std::FeagiWebSocketClientRequesterProperties;
    use feagi_io::protocol_implementations::zmq::FeagiZmqClientRequesterProperties;

    let args: Vec<String> = std::env::args().collect();
    if args.len() != 4 {
        eprintln!(
            "Usage: {} <registration_endpoint> <heartbeat_interval_ms> <poll_interval_ms>",
            args.get(0).map(String::as_str).unwrap_or("psuedo_example")
        );
        eprintln!("Examples:");
        eprintln!("  tcp://<host>:<port> (ZMQ)");
        eprintln!("  <host>:<port> or ws://<host>:<port> (WebSocket)");
        return Ok(());
    }

    let registration_endpoint = &args[1];
    let heartbeat_interval_ms: u64 = args[2].parse()?;
    let poll_interval_ms: u64 = args[3].parse()?;

    let registration_props: Box<
        dyn feagi_io::traits_and_enums::client::FeagiClientRequesterProperties,
    > = if registration_endpoint.starts_with("tcp://")
        || registration_endpoint.starts_with("ipc://")
        || registration_endpoint.starts_with("inproc://")
        || registration_endpoint.starts_with("pgm://")
        || registration_endpoint.starts_with("epgm://")
    {
        Box::new(FeagiZmqClientRequesterProperties::new(
            registration_endpoint,
        )?)
    } else {
        Box::new(FeagiWebSocketClientRequesterProperties::new(
            registration_endpoint,
        )?)
    };

    let agent_descriptor = AgentDescriptor::new("neuraville", "example_agent", 1)?;
    let auth_token = AuthToken::new([0u8; 32]);

    let driver = TokioDriverConfig {
        poll_interval: std::time::Duration::from_millis(poll_interval_ms),
        timing: SessionTimingConfig {
            heartbeat_interval_ms,
            registration_deadline_ms: None,
        },
        sensory_rate_negotiation: None,
    };

    let mut agent = TokioEmbodimentAgent::new_connect_and_register(
        registration_props,
        agent_descriptor,
        auth_token,
        vec![
            AgentCapabilities::SendSensorData,
            AgentCapabilities::ReceiveMotorData,
        ],
        driver,
    )
    .await?;

    // Application-specific loop would go here.
    // For demonstration, drive a few maintenance ticks.
    for _ in 0..3 {
        agent.tick()?;
        tokio::time::sleep(std::time::Duration::from_millis(poll_interval_ms)).await;
    }

    Ok(())
}
