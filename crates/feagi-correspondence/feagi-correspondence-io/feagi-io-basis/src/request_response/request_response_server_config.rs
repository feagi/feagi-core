use crate::request_response::request_response_server::RequestResponseServer;

pub trait RequestResponseServerConfig {
    type Server: RequestResponseServer;

    fn run_server(self) -> Result<std::thread::JoinHandle<Self::Server>, ()>; // TODO Error Checking

}