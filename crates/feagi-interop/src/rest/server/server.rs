use picoserve::Router;
use picoserve::routing::PathRouter;
use feagi_basis::generic_thread_worker::{GenericThreadWorker, GenericThreadWorkerCommand, GenericThreadWorkerSpawner};
use crate::error::FeagiInteropError;
use crate::rest::server::error::FeagiRestServerError;

pub struct RestServer {
    #[cfg(feature = "std")]
    worker: GenericThreadWorker
    // TODO no std
}

impl RestServer {
    #[cfg(feature = "std")]
    pub async fn start_rest_server<RestEndpoints>(socket_address: core::net::SocketAddr) -> Result<Self, FeagiRestServerError> {
        let worker = GenericThreadWorker::new_and_start(
            RestServerThreadSpawner {socket_address}
        )?;
        Ok(Self { worker })
    }

    pub async fn stop_rest_server(self) -> () {
        #[cfg(feature = "std")]
        self.worker.stop_thread().await
    }
}


#[cfg(feature = "std")]
struct RestServerThreadSpawner {
    pub socket_address: core::net::SocketAddr
}

#[cfg(feature = "std")]
impl GenericThreadWorkerSpawner for RestServerThreadSpawner {
    type ThreadSpawnError = FeagiRestServerError;

    fn spawn_thread_worker(
        self,
        command_receiver: thingbuf::mpsc::Receiver<GenericThreadWorkerCommand<()>>
    ) -> Result<std::thread::JoinHandle<()>, Self::ThreadSpawnError> {

        let socket = std::net::TcpListener::bind(self.socket_address).map_err(|_| FeagiRestServerError::FailedToBindToSocket)?;

        static CONFIG: picoserve::Config = picoserve::Config::const_default().close_connection_after_response();
        let mut router = picoserve::Router::new().with_state(()); // TODO actual state later

        configure_router(&mut router);

        let handle = std::thread::spawn(loop_rest_server(socket, router, command_receiver));
        Ok(handle)
    }
}

#[cfg(feature = "std")]
async fn loop_rest_server<RestEndpoints>(
    socket: std::net::TcpListener,
    router: Router<impl PathRouter>,
    command_receiver: thingbuf::mpsc::Receiver<GenericThreadWorkerCommand<()>>)
    -> Result<(), FeagiRestServerError> {
    
    loop {
        match socket.accept() {
            Ok((stream, client_socket)) => match picoserve::Server::serve(client_socket, router) {

            },
            Err(_) => return Err(FeagiRestServerError::ServerIncomingDataParseFailure), // TODO ????
        }
    }
}

fn configure_router(router: &mut picoserve::Router<impl PathRouter>) {
    // TODO Open API

    // TODO swagger

    // TODO state
}
