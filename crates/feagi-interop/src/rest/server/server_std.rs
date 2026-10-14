//! REST server routes on std

use crate::rest::server::error::FeagiRestServerError;
use crate::rest::server::server_common::{build_router_with_routes, FeagiRestServerWorker, HTTP_BUFFER_LEN};
use core::future::Future;
use feagi_basis::parallelism::feagi_channel::channel_error::FeagiChannelReceiveError;
use feagi_basis::parallelism::feagi_worker::feagi_worker::{FeagiWorkerHandleDynamic, FeagiWorkerNotificationsDynamic};
use feagi_basis::parallelism::feagi_worker::notifications::{FeagiWorkerNotification, FeagiWorkerReply};
use feagi_basis::parallelism::feagi_worker::std_thread_spawner::FeagiStdThreadSpawner;

/// One request per TCP connection, matching a single accept loop on this thread.
static CONFIG: picoserve::Config = picoserve::Config::const_default().close_connection_after_response();

/// Notifications queued for the server worker. Only `Stop` is ever sent
const NOTIFICATION_QUEUE_CAPACITY: usize = 1;

/// The server worker takes no custom notifications and gives no custom replies
type RestServerWorkerHandle = FeagiWorkerHandleDynamic<(), (), std::thread::JoinHandle<()>>;
type RestServerNotifications = FeagiWorkerNotificationsDynamic<(), ()>;

/// HTTP server for the OpenAPI document and, when enabled, the Swagger UI.
pub struct RestServer {
    worker: RestServerWorkerHandle,
}

/// What the std REST server needs to start
pub struct RestServerStartParameters {
    pub socket_address: core::net::SocketAddr,
}

impl FeagiRestServerWorker for RestServer {
    type StartParameters = RestServerStartParameters;

    /// Binds `socket_address` on the calling thread, then starts the listener thread.
    fn start_rest_server(parameters: Self::StartParameters) -> Result<Self, FeagiRestServerError> {
        let listener = std::net::TcpListener::bind(parameters.socket_address).map_err(|_| FeagiRestServerError::FailedToBindToSocket)?;
        let worker = RestServerWorkerHandle::start(&FeagiStdThreadSpawner, NOTIFICATION_QUEUE_CAPACITY, |notifications| {
            move || run_rest_server(listener, notifications)
        })
        .map_err(|_| FeagiRestServerError::FailedToStartWorker)?;
        Ok(Self { worker })
    }

    /// Asks the listener thread to stop after the current connection, waits for it to
    /// acknowledge, then joins it.
    ///
    /// A stop sent while the thread is blocked in `accept` is observed when that call returns.
    async fn stop_rest_server(mut self) -> Result<(), FeagiRestServerError> {
        self.worker.stop().await?;
        self.worker.join().map_err(|_| FeagiRestServerError::WorkerPanicked)
    }
}

/// Accepts one connection at a time and serves it to completion. Checks for notifications
/// before each accept, and exits after acknowledging `Stop` or once the handle is dropped.
async fn run_rest_server(listener: std::net::TcpListener, notifications: RestServerNotifications) {
    let router = route_brain_visualizer_startup(build_router_with_routes());
    let mut running = true;

    while running {
        let notification_result = notifications
            .try_process_next(async |notification| match notification {
                FeagiWorkerNotification::Stop => {
                    running = false;
                    FeagiWorkerReply::Acknowledged
                }
                // No custom notifications are defined
                FeagiWorkerNotification::Custom(()) | FeagiWorkerNotification::Nothing => FeagiWorkerReply::Nothing,
            })
            .await;
        match notification_result {
            Ok(()) | Err(FeagiChannelReceiveError::ChannelEmpty) => {}
            // The handle was dropped, nothing can stop this worker anymore
            Err(_) => break,
        }
        if !running {
            break;
        }

        let (stream, _peer) = match listener.accept() {
            Ok((stream, _peer)) => (stream, _peer),
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue, // try again
            Err(_error) => break,
        };

        let Some(socket) = StdThreadSocket::from_accepted(stream) else {
            continue;
        };

        let mut http_buffer = [0u8; HTTP_BUFFER_LEN];
        let serve = picoserve::Server::custom(&router, StdThreadTimer, &CONFIG, &mut http_buffer).serve(socket);
        // One failed request does not stop the listener.
        let _serve_result = serve.await;
    }
}

//region patching

const JSON_CONTENT_TYPE: &str = "application/json";

const HEALTH_CHECK_JSON: &str = r#"{"genome_availability":true,"brain_readiness":true,"genome_validity":true,"feagi_session":1,"genome_num":1,"burst_engine":true,"simulation_timestep":0.05,"influxdb_availability":false,"neuron_count_max":0,"synapse_count_max":0,"neuron_count":0,"synapse_count":0}"#;

/// Brain Visualizer reads this body with `String.to_float()`, so the body is a JSON number.
const SIMULATION_TIMESTEP_JSON: &str = "0.05";

/// Brain Visualizer reads this body with `String.to_int()`, so the body is a JSON number.
const VISUALIZATION_SKIP_RATE_JSON: &str = "1";

/// Brain Visualizer reads this body with `String.to_int()`, so the body is a JSON number.
const VISUALIZATION_SUPPRESSION_THRESHOLD_JSON: &str = "1";

/// Brain Visualizer reads this body with `String.to_int()`, so the body is a JSON number.
const PLASTICITY_QUEUE_DEPTH_JSON: &str = "1";

const CONNECTION_INFO_JSON: &str =
    r#"{"websocket":{"enabled":true,"host":"127.0.0.1","endpoints":{"registration":"ws://127.0.0.1:9050","visualization":"ws://127.0.0.1:9050"}}}"#;

const CORTICAL_AREA_GEOMETRY_JSON: &str = "{}";

const MORPHOLOGIES_JSON: &str = "{}";

const CORTICAL_MAP_DETAILED_JSON: &str = "{}";

/// One root region and no cortical areas. `parent_region_id` null marks the root.
const REGIONS_MEMBERS_JSON: &str =
    r#"{"root":{"title":"root","coordinate_2d":[0,0],"coordinate_3d":[0,0,0],"regions":[],"areas":[],"parent_region_id":null}}"#;

const IPU_TYPES_JSON: &str = "{}";

const OPU_TYPES_JSON: &str = "{}";

const AGENT_CAPABILITIES_JSON: &str = "{}";


fn route_brain_visualizer_startup<R: picoserve::routing::PathRouter>(
    router: picoserve::Router<R>,
) -> picoserve::Router<impl picoserve::routing::PathRouter> {
    use picoserve::response::File;
    use picoserve::routing::get_service;

    fn json_file(body: &'static str) -> File {
        File::with_content_type(JSON_CONTENT_TYPE, body.as_bytes())
    }

    router
        .route("/v1/system/health_check", get_service(json_file(HEALTH_CHECK_JSON)))
        .route("/v1/burst_engine/simulation_timestep", get_service(json_file(SIMULATION_TIMESTEP_JSON)))
        .route("/v1/network/connection_info", get_service(json_file(CONNECTION_INFO_JSON)))
        .route(
            "/v1/cortical_area/cortical_area/geometry",
            get_service(json_file(CORTICAL_AREA_GEOMETRY_JSON)),
        )
        .route("/v1/morphology/morphologies", get_service(json_file(MORPHOLOGIES_JSON)))
        .route(
            "/v1/cortical_area/cortical_map_detailed",
            get_service(json_file(CORTICAL_MAP_DETAILED_JSON)),
        )
        .route("/v1/region/regions_members", get_service(json_file(REGIONS_MEMBERS_JSON)))
        .route("/v1/cortical_area/ipu/types", get_service(json_file(IPU_TYPES_JSON)))
        .route("/v1/cortical_area/opu/types", get_service(json_file(OPU_TYPES_JSON)))
        .route(
            "/v1/system/cortical_area_visualization_skip_rate",
            get_service(json_file(VISUALIZATION_SKIP_RATE_JSON)),
        )
        .route(
            "/v1/system/cortical_area_visualization_suppression_threshold",
            get_service(json_file(VISUALIZATION_SUPPRESSION_THRESHOLD_JSON)),
        )
        .route(
            "/v1/neuroplasticity/plasticity_queue_depth",
            get_service(json_file(PLASTICITY_QUEUE_DEPTH_JSON)),
        )
        .route("/v1/agent/capabilities/all", get_service(json_file(AGENT_CAPABILITIES_JSON)))
}

//endregion

//region Internal Members

//region Picoserver Timer
/// Marker paired with [`StdThreadTimer`] and [`StdThreadSocket`].
struct StdThreadRuntime;

/// Timer whose delay never sleeps.
///
/// picoserve polls this before the socket read. Sleeping here would stall every request for
/// the full timeout. Deadlines are the socket's read and write timeouts instead.
struct StdThreadTimer;

impl picoserve::time::Timer<StdThreadRuntime> for StdThreadTimer {
    fn delay(&self, _duration: picoserve::time::Duration) -> impl Future<Output = ()> {
        core::future::pending()
    }
}
//endregion

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
struct StdThreadIoError(std::io::Error);

impl picoserve::io::Error for StdThreadIoError {
    fn kind(&self) -> picoserve::io::ErrorKind {
        match self.0.kind() {
            std::io::ErrorKind::NotFound => picoserve::io::ErrorKind::NotFound,
            std::io::ErrorKind::PermissionDenied => picoserve::io::ErrorKind::PermissionDenied,
            std::io::ErrorKind::ConnectionRefused => picoserve::io::ErrorKind::ConnectionRefused,
            std::io::ErrorKind::ConnectionReset => picoserve::io::ErrorKind::ConnectionReset,
            std::io::ErrorKind::ConnectionAborted => picoserve::io::ErrorKind::ConnectionAborted,
            std::io::ErrorKind::NotConnected => picoserve::io::ErrorKind::NotConnected,
            std::io::ErrorKind::AddrInUse => picoserve::io::ErrorKind::AddrInUse,
            std::io::ErrorKind::AddrNotAvailable => picoserve::io::ErrorKind::AddrNotAvailable,
            std::io::ErrorKind::BrokenPipe => picoserve::io::ErrorKind::BrokenPipe,
            std::io::ErrorKind::AlreadyExists => picoserve::io::ErrorKind::AlreadyExists,
            std::io::ErrorKind::InvalidInput => picoserve::io::ErrorKind::InvalidInput,
            std::io::ErrorKind::InvalidData => picoserve::io::ErrorKind::InvalidData,
            std::io::ErrorKind::TimedOut => picoserve::io::ErrorKind::TimedOut,
            std::io::ErrorKind::Interrupted => picoserve::io::ErrorKind::Interrupted,
            std::io::ErrorKind::Unsupported => picoserve::io::ErrorKind::Unsupported,
            std::io::ErrorKind::OutOfMemory => picoserve::io::ErrorKind::OutOfMemory,
            std::io::ErrorKind::WriteZero => picoserve::io::ErrorKind::WriteZero,
            _ => picoserve::io::ErrorKind::Other,
        }
    }
}

/// Repeats `operation` when the OS reports that it was interrupted.
fn retry_interrupted<T>(mut operation: impl FnMut() -> std::io::Result<T>) -> std::io::Result<T> {
    loop {
        match operation() {
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            result => return result,
        }
    }
}

//region Socket

/// Blocking `std::net` socket split into a read handle and a write handle.
struct StdThreadSocket {
    read: std::net::TcpStream,
    write: std::net::TcpStream,
}

impl StdThreadSocket {
    /// Clones `stream` so the two halves can block independently, and arms its deadlines.
    ///
    /// Returns `None` when the socket cannot be configured. The caller drops that connection.
    fn from_accepted(stream: std::net::TcpStream) -> Option<Self> {
        let read_timeout = std::time::Duration::from_millis(CONFIG.timeouts.read_request.as_millis());
        let write_timeout = std::time::Duration::from_millis(CONFIG.timeouts.write.as_millis());
        stream.set_read_timeout(Some(read_timeout)).ok()?;
        stream.set_write_timeout(Some(write_timeout)).ok()?;
        let write = stream.try_clone().ok()?;
        Some(Self { read: stream, write })
    }
}

impl picoserve::io::Socket<StdThreadRuntime> for StdThreadSocket {
    type Error = StdThreadIoError;
    type ReadHalf<'a> = StdThreadReadHalf<'a>;
    type WriteHalf<'a> = StdThreadWriteHalf<'a>;

    fn split(&mut self) -> (Self::ReadHalf<'_>, Self::WriteHalf<'_>) {
        (
            StdThreadReadHalf { stream: &mut self.read },
            StdThreadWriteHalf { stream: &mut self.write },
        )
    }

    async fn abort<T: picoserve::Timer<StdThreadRuntime>>(
        self,
        _timeouts: &picoserve::Timeouts,
        _timer: &T,
    ) -> Result<(), picoserve::Error<Self::Error>> {
        retry_interrupted(|| self.read.shutdown(std::net::Shutdown::Both)).map_err(|error| picoserve::Error::Write(StdThreadIoError(error)))
    }

    async fn shutdown<T: picoserve::Timer<StdThreadRuntime>>(
        mut self,
        _timeouts: &picoserve::Timeouts,
        _timer: &T,
    ) -> Result<(), picoserve::Error<Self::Error>> {
        retry_interrupted(|| std::io::Write::flush(&mut self.write)).map_err(|error| picoserve::Error::Write(StdThreadIoError(error)))?;
        retry_interrupted(|| self.write.shutdown(std::net::Shutdown::Write)).map_err(|error| picoserve::Error::Write(StdThreadIoError(error)))?;

        let mut buffer = [0u8; 128];
        loop {
            match retry_interrupted(|| std::io::Read::read(&mut self.read, &mut buffer)) {
                Ok(0) => break,
                Ok(_) => continue,
                // The response is already written. The read deadline ends the drain.
                Err(error) if error.kind() == std::io::ErrorKind::TimedOut => break,
                Err(error) => return Err(picoserve::Error::Read(StdThreadIoError(error))),
            }
        }
        Ok(())
    }
}

struct StdThreadReadHalf<'a> {
    stream: &'a mut std::net::TcpStream,
}

impl picoserve::io::ErrorType for StdThreadReadHalf<'_> {
    type Error = StdThreadIoError;
}

impl picoserve::io::Read for StdThreadReadHalf<'_> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        retry_interrupted(|| std::io::Read::read(self.stream, buf)).map_err(StdThreadIoError)
    }
}

struct StdThreadWriteHalf<'a> {
    stream: &'a mut std::net::TcpStream,
}

impl picoserve::io::ErrorType for StdThreadWriteHalf<'_> {
    type Error = StdThreadIoError;
}

impl picoserve::io::BaseWrite for StdThreadWriteHalf<'_> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        retry_interrupted(|| std::io::Write::write(self.stream, buf)).map_err(StdThreadIoError)
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        retry_interrupted(|| std::io::Write::flush(self.stream)).map_err(StdThreadIoError)
    }
}

impl picoserve::io::Write for StdThreadWriteHalf<'_> {
    async fn write_with<F: FnOnce(picoserve::mem::BorrowedCursor<'_>) -> R, R>(&mut self, f: F) -> Result<R, Self::Error> {
        let mut buffer = [0u8; 1024];
        let mut buffer = picoserve::mem::BorrowedBuffer::new(&mut buffer);
        let output = f(buffer.unfilled());
        retry_interrupted(|| std::io::Write::write_all(self.stream, buffer.filled()))
            .map(|()| output)
            .map_err(StdThreadIoError)
    }
}

//endregion

//endregion
