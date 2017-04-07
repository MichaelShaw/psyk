#![allow(dead_code)]

extern crate psyk;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;


extern crate bytes;
extern crate futures;
#[macro_use]
extern crate tokio_core;
extern crate tokio_io;

use std::env;

use std::net::SocketAddr;

use futures::sync::mpsc::{self, UnboundedSender};

use psyk::server::ServerHandle;

use std::thread;

#[derive(Serialize, Deserialize, Clone)]
struct ClientEvent {
    pub msg : String,
}

#[derive(Serialize, Deserialize, Clone)] 
struct ServerEvent {
    pub msg : String
}

fn main() {
    let addr = env::args().nth(1).unwrap_or("127.0.0.1:8080".to_string());
    let addr : SocketAddr = addr.parse().unwrap();


    let (sender, receiver) = mpsc::unbounded();

    let server_handle : ServerHandle<ServerEvent, ClientEvent> = ServerHandle {
        sender : sender,
    };
    
    let poison_pill = psyk::server::run_server(server_handle, addr).unwrap();

    println!("alright, we made it!");

    let res = poison_pill.shutdown();

    println!("ok we even shutdown -> {:?}", res);





    // handle.spawn(srv.map_err(|e| panic!("srv error: {}", e)));
}
