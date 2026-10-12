use std::thread::JoinHandle;
use picoserve::Router;
use crate::error::FeagiInteropError;

pub struct RestServer {
    join_handle: JoinHandle<Result<(), FeagiRestServerError>>,
    command_channel: RequestChannelPooled
}

impl RestServer {

    #[cfg(feature = "std")]
    pub async fn new_server<RestEndpoints>(socket_address: core::net::SocketAddr) -> Result<Self, FeagiInteropError> {

        let (requester, responder) = create_requester_and_responder_pooled::<RestServerCommands, (), 1 , false>(1);

        static CONFIG: picoserve::Config = picoserve::Config::const_default().close_connection_after_response();

        let socket = std::net::TcpListener::bind(socket_address)
            .map_err(|_| FeagiInteropError::FailedToBindSocket)?;

        let mut router = picoserve::Router::new()
            .with_state();

        /*
        #[cfg(feature = "transport_rest_swagger")]
        {
            router.route("/", picoserve::routing::get(async || ))
        }
        
         */

        // TODO add routes for Rest Endpoints (macro)

        let handle = std::thread::spawn(loop_rest_server(socket, router));

        Ok(Self { 
            join_handle: handle,
            command_channel: requester
        })
    }
    
    // TODO no-std variant
    
    // TODO Stop command
}



#[repr(u8)]
#[derive(Clone, Copy, Debug)]
enum RestServerCommands {
    Start, // is this ever used?
    Stop
    // TODO others?
    // TODO implement this, right now the socket check is blocking so we dont check the loop
}

#[cfg(feature = "std")]
async fn loop_rest_server<RestEndpoints>(socket: std::net::TcpListener, router: Router<>) -> Result<(), FeagiRestServerError> {
    loop {
        match socket.accept() {
            Ok((stream, client_socket)) => {
                match picoserve::Server::serve(client_socket, router) {

                }
            }
            Err(_) => return Err(FeagiRestServerError::ServerParseFailure), // TODO ????
        }
    }
}