//! REST server routes on std

use core::marker::PhantomData;
use core::future::Future;
use feagi_basis::threading::generic_thread_worker::{GenericThreadWorker, GenericThreadWorkerNotification, GenericThreadWorkerSpawner};
use feagi_basis::threading::block_on_thread;
use crate::rest::server::error::FeagiRestServerError;
use crate::rest::server::server_common::{build_router_with_routes, HTTP_BUFFER_LEN};

type RequestPathDefinition = ();


/// One request per TCP connection, matching a single accept loop on this thread.
static CONFIG: picoserve::Config = picoserve::Config::const_default().close_connection_after_response();

/// HTTP server for the OpenAPI document and, when enabled, the Swagger UI.
pub struct RestServer {
    worker: GenericThreadWorker,
}

impl RestServer {
    /// Binds `socket_address` and starts the listener thread.
    pub async fn start_rest_server(socket_address: core::net::SocketAddr) -> Result<Self, FeagiRestServerError> {
        let worker = GenericThreadWorker::new_and_start(RestServerThreadSpawner {
            socket_address,
            _routes: PhantomData::<RequestPathDefinition>,
        })?;
        Ok(Self { worker })
    }

    /// Asks the listener thread to stop after the current connection, then joins it.
    ///
    /// A stop sent while the thread is blocked in `accept` is observed when that call returns.
    pub async fn stop_rest_server(self) {
        self.worker.stop_thread().await
    }
}

struct RestServerThreadSpawner {
    socket_address: core::net::SocketAddr,
    _routes: PhantomData<RequestPathDefinition>,
}

impl GenericThreadWorkerSpawner for RestServerThreadSpawner {
    type ThreadSpawnError = FeagiRestServerError;

    fn spawn_thread_worker(
        self,
        command_receiver: thingbuf::mpsc::Receiver<GenericThreadWorkerNotification<()>>,
    ) -> Result<std::thread::JoinHandle<()>, Self::ThreadSpawnError> {
        let listener = std::net::TcpListener::bind(self.socket_address).map_err(|_| FeagiRestServerError::FailedToBindToSocket)?;

        let handle = std::thread::spawn(move || {
            let _routes = self._routes;
            run_rest_server(listener, command_receiver);
        });
        Ok(handle)
    }
}

/// Accepts one connection at a time and serves it to completion.
fn run_rest_server(listener: std::net::TcpListener, command_receiver: thingbuf::mpsc::Receiver<GenericThreadWorkerNotification<()>>) {

    let router = build_router_with_routes();

    loop {

        let notification: GenericThreadWorkerNotification<()> = command_receiver.try_recv().into();
        match notification {
            GenericThreadWorkerNotification::Nothing => {} // do nothing
            GenericThreadWorkerNotification::Custom(_) => {} // No custom commands defined, so ignore this
            GenericThreadWorkerNotification::Stop => {break;}
            GenericThreadWorkerNotification::ChannelClosed => {break;}
        }

        let (stream, _peer) = match listener.accept() {
            Ok((stream, _peer )) => (stream, _peer),
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue, // try again
            Err(_error) => break,
        };

        let Some(socket) = StdThreadSocket::from_accepted(stream) else {
            continue;
        };

        let mut http_buffer = [0u8; HTTP_BUFFER_LEN];
        let serve = picoserve::Server::custom(&router, StdThreadTimer, &CONFIG, &mut http_buffer).serve(socket);
        // One failed request does not stop the listener.
        let _serve_result = block_on_thread(serve);
    }
}

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

    async fn abort<T: picoserve::Timer<StdThreadRuntime>>(self, _timeouts: &picoserve::Timeouts, _timer: &T) -> Result<(), picoserve::Error<Self::Error>> {
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


