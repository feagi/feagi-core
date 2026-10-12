use picoserve::Router;
use feagi_basis::generic_thread_worker::{GenericThreadWorker, GenericThreadWorkerCommand, GenericThreadWorkerSpawner};
use crate::error::FeagiInteropError;


pub struct RestServer {
    #[cfg(feature = "std")]
    worker: GenericThreadWorker
    // TODO no std
}

impl RestServer {
    #[cfg(feature = "std")]
    pub async fn start_rest_server<RestEndpoints>(socket_address: core::net::SocketAddr) -> Result<Self, FeagiInteropError> {
        let worker = GenericThreadWorker::new_and_start(
            RestServerThreadSpawner {socket_address}
        )?;
        Self { worker }
    }
}


#[cfg(feature = "std")]
struct RestServerThreadSpawner {
    pub socket_address: core::net::SocketAddr
}

#[cfg(feature = "std")]
impl GenericThreadWorkerSpawner for RestServerThreadSpawner {
    type ThreadSpawnError = FeagiInteropError;

    fn spawn_thread_worker(
        self,
        command_receiver: thingbuf::mpsc::Receiver<GenericThreadWorkerCommand<CustomCommand>>
    ) -> Result<std::thread::JoinHandle<FinalResult>, Self::ThreadSpawnError> {

        let socket = std::net::TcpListener::bind(socket_address).map_err(|_| FeagiInteropError::FailedToBindSocket)?;

        static CONFIG: picoserve::Config = picoserve::Config::const_default().close_connection_after_response();
        let mut router = picoserve::Router::new().with_state();

        // TODO Open API, SWAGGER

        // TODO state

        let handle = std::thread::spawn(loop_rest_server(socket, router));
        Ok(handle)
    }
}

#[cfg(feature = "std")]
async fn loop_rest_server<RestEndpoints>(socket: std::net::TcpListener, router: Router) -> Result<(), FeagiRestServerError> {
    loop {
        match socket.accept() {
            Ok((stream, client_socket)) => match picoserve::Server::serve(client_socket, router) {

            },
            Err(_) => return Err(FeagiRestServerError::ServerParseFailure), // TODO ????
        }
    }
}
