// TODO we can pick different ones for different usecases

mod default {
    use feagi_io_basis::create_request_response_sender_receiver_sets;
    use feagi_io_basis::contracts::request_response::feagi_server_requests::agent::*;
    use feagi_io_basis::contracts::request_response::feagi_server_requests::system::*;


    create_request_response_sender_receiver_sets! (
        RestSTD: {
            (agent, 10, true, 10),
        }
    );

}

