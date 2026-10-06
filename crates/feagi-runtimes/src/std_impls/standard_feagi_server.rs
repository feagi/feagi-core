//! Module for running a FEAGI server in an STD environment

use feagi_correspondence::feagi_correspondence_io::create_request_response_sender_receiver_sets;
use feagi_correspondence::feagi_correspondence_io::feagi_servers::FeagiServers;

pub fn launch_standard_feagi_server() -> std::thread::JoinHandle<()> {
    
    let mut servers = FeagiServers::new();

    create_request_response_sender_receiver_sets! {
        WebServerEndpoints: {
            (
                agent, 10, true, 10
            )
        }
    }
    
    servers.start_web_server::<>(/* requests */, /* SocketAddrV4 */);
}




