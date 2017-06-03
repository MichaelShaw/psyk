#![allow(dead_code)]

extern crate psyk;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;



extern crate bytes;
extern crate futures;
// #[macro_use]
extern crate tokio_core;
extern crate tokio_io;

use std::env;


use std::net::SocketAddr;

use psyk::server::ServerEventHandler;

use std::thread;
use std::thread::JoinHandle;

use std::sync::mpsc;

#[derive(Serialize, Deserialize, Clone, Debug)]
enum MathToServerEvent {
    Get,
    Add(u32),
    Sub(u32),
    Mul(u32),
    Div(u32),
}

#[derive(Serialize, Deserialize, Clone, Debug)] 
enum MathToClientEvent {
    Val(u32),
}


fn main() {
    let addr = env::args().nth(1).unwrap_or("127.0.0.1:8080".to_string());
    let addr : SocketAddr = addr.parse().unwrap();

    let (server_event_handler, join_handle) = spawn_math_server();

    // let server = server_event_handler.clone();


    let poison_pill = psyk::server::run_server(server_event_handler.clone(), addr).unwrap();

    println!("Main :: alright, server started, sending the damn thing");

    server_event_handler.sender.send(psyk::server::ServerInboundEvent::ClientMessage { address: addr, event : MathToServerEvent::Get }).unwrap();

    let res = poison_pill.shutdown();

    println!("Main :: ok we even shutdown -> {:?}", res);


    // let (client_tx, client_rx) = mpsc::channel();

    // let (poison_pill, server_tx) = psyk::client::run_client::(client_tx, addr).unwrap();



}

fn spawn_math_client() {

    let join_handle = thread::spawn(move || {

        45
    });

}

fn spawn_math_server() -> (ServerEventHandler<MathToServerEvent, MathToClientEvent>, JoinHandle<u32>) {
    use std::collections::HashMap;
    
    let (sender, receiver) = mpsc::channel();

    let join_handle = thread::spawn(move || {
        let mut val = 0_u32;

        println!("Server :: math server entering main loop");

        let mut clients = HashMap::new();

        use psyk::server::ServerInboundEvent::*;
        use MathToServerEvent::*;

        loop {
            match receiver.recv() {
                Ok(event) => {
                    match event {
                        FailureToBind { address } => {
                            println!("Server :: failure to bind on address -> {:?}", address);
                            break;
                        },
                        ServerFinished => {
                            println!("Server :: server process finished");
                            break;
                        }
                        ClientConnected { address, client_sender } => {
                            println!("Server :: client @ {:?} connected :D", address);
                            clients.insert(address, client_sender);
                        },
                        ClientMessage { address, event } => {
                            println!("Server :: received client message -> {:?} from {:?}", event, address);
                            match event {
                                Get => (),
                                Add(n) => val += n,
                                Sub(n) => val -= n,
                                Mul(n) => val *= n,
                                Div(n) => val /= n,
                            }

                            if let Some(sender) = clients.get(&address) {
                                println!("Server :: sending current value {} to client {:?}", val, address);
                                sender.send(MathToClientEvent::Val(val)).unwrap();
                            } else {
                                println!("Server :: we're not aware of client {:?}", address);
                            }
                        },
                        ClientDisconnected { address } => {
                            println!("Server :: client disconnected {:?}", address);
                            clients.remove(&address);
                        },
                    }
                },
                Err(e) => {
                    println!("Server :: problem receiving event :-( {:?}", e);
                    break;
                },
            }
        }
        //     println!("math server got message -> {:?}", event);
  
       

        println!("Server :: math done");


        32
    });

    (ServerEventHandler {
        sender : sender,
    }, join_handle)
}


