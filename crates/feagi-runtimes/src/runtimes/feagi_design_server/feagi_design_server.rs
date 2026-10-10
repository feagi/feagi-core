
// TODO use generics to allow different kinds of server
// TODO config struct type (also using generics)
// TODO shutdown result, server thread launch result
// TODO input channel for commands

pub struct FeagiDesignServer {
    agent_servers: (),
    feagi_brain: (),
    system_configuration: ()
}

/*
impl FeagiDesignServer {
    
    pub fn launch_server_thread(/*TODO config*/) -> std::thread::JoinHandle<()> {
        
        
        let server = Self {
            agent_servers: (),
            feagi_brain: (),
            system_configuration: (),
        };
        
        let handle = std::thread::spawn( );

        handle
    }
    
    
    fn server_loop(&mut self) {
        
    }
}





fn feagi_server_loop(mut server: FeagiDesignServer) -> () {
    loop {
        server.server_loop()
    }
}

 */