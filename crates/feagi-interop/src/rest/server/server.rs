

const fn configure_router<RequestPathDefinition>() -> picoserve::Router<impl picoserve::routing::PathRouter> {

    static CONFIG: picoserve::Config = picoserve::Config::const_default().close_connection_after_response();
    let mut router = picoserve::Router::new().with_state(()); // TODO actual state later

    #[cfg(feature = "open_api_endpoint")]
    const OPEN_API_JSON: &str = {

    };

    #[cfg(feature = "swagger_web_ui")]
    {

    }

    // TODO state
}



#[cfg(feature = "std")]
pub use impl_std::RestServer;


#[cfg(feature = "std")]
mod impl_std {
    use picoserve::doctests_utils::router;
    use picoserve::Router;
    use picoserve::routing::PathRouter;
    use feagi_basis::generic_thread_worker::{GenericThreadWorker, GenericThreadWorkerCommand, GenericThreadWorkerSpawner};
    use crate::rest::server::error::FeagiRestServerError;
    use crate::rest::server::server::{configure_router};

    pub struct RestServer {
        worker: GenericThreadWorker
    }

    impl RestServer {
        pub async fn start_rest_server<RequestPathDefinition>(socket_address: core::net::SocketAddr) -> Result<Self, FeagiRestServerError> {
            let worker = GenericThreadWorker::new_and_start(
                RestServerThreadSpawner {socket_address}
            )?;
            Ok(Self { worker })
        }

        pub async fn stop_rest_server(self) -> () {
            self.worker.stop_thread().await
        }
    }

    struct RestServerThreadSpawner {
        pub socket_address: core::net::SocketAddr
    }

    impl GenericThreadWorkerSpawner for RestServerThreadSpawner {
        type ThreadSpawnError = FeagiRestServerError;

        fn spawn_thread_worker(
            self,
            command_receiver: thingbuf::mpsc::Receiver<GenericThreadWorkerCommand<()>>
        ) -> Result<std::thread::JoinHandle<()>, Self::ThreadSpawnError> {

            let socket = std::net::TcpListener::bind(self.socket_address).map_err(|_| FeagiRestServerError::FailedToBindToSocket)?;

            configure_router();

            let handle = std::thread::spawn(loop_rest_server(socket, router, command_receiver));
            Ok(handle)
        }
    }

    async fn loop_rest_server<RequestPathDefinition>(
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
}






