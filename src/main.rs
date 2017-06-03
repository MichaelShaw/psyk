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

use psyk::client::ClientEventHandler;
use psyk::server::ServerEventHandler;

use std::{thread, time};
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

    let (server_event_handler, server_join_handle) = spawn_math_server();

    println!("Main :: Server Event Handler Started");
    let server_poison_pill = psyk::server::run_server(server_event_handler.clone(), addr).unwrap();

    println!("Main :: Server TCPListener Started");
    thread::sleep(time::Duration::from_millis(100));

    let (client_event_handler, client_join_handler) = spawn_math_client(addr);
    println!("Main :: Client Event Handler Started");


    println!("Waiting for Client to shutdown");
    let client_res = client_join_handler.join();

    // let server = server_event_handler.clone();

    let res = server_poison_pill.shutdown();
    

    
    

    println!("Main :: ok we even shutdown -> {:?}", res);


    // let (client_tx, client_rx) = mpsc::channel();

    // let (poison_pill, server_tx) = psyk::client::run_client::(client_tx, addr).unwrap();



}

fn spawn_math_client(server_address: SocketAddr) -> (ClientEventHandler<MathToClientEvent, MathToServerEvent>, JoinHandle<u32>) {
    use psyk::client::ClientInboundEvent::*;

    let (sender, receiver) = mpsc::channel();

    // state for server

    let client_event_handler = ClientEventHandler {
        sender : sender,
    };

    let handler_copy = client_event_handler.clone();


    let join_handle = thread::spawn(move || {
        println!("Client :: connecting to {:?}", server_address);
        // attempt to connect to server
        let client_poison_pill = psyk::client::run_client(client_event_handler.clone(), server_address).unwrap();

        let mut to_server : Option<psyk::client::ChannelToServer<MathToServerEvent>> = None;

        loop {
            match receiver.recv() {
                Ok(event) => {
                    match event {
                        FailedToConnect { address } => {
                            println!("Client :: failed to connect to {:?}", address);
                            break;
                        },
                        ServerConnected { address, channel_to_server } => {
                            println!("Client :: connected to server @ {:?}", address);
                            // psyk::client::ClientOutboundEvent::SendMessage { event: MathToServerEvent::Get }
                            channel_to_server.sender.send(MathToServerEvent::Get);

                            println!("Client :: Sending get");
                            to_server = Some(channel_to_server)
                            
                        }, // that is NOT good enough ..
                        ServerMessage { address, event } => {
                            println!("Client :: received event {:?} from server @ {:?}", event, address);
                            break;
                        },
                        ServerDisconnected { address } => {
                            println!("Client :: server disconnected @ {:?}", address);
                            break;
                        },
                        ClientFinished { address } => {
                            println!("Client :: finished event");
                            break;
                        }
                    }

                },
                Err(e) => {
                    println!("Client :: problem receiving event :-( {:?}", e);
                    break;
                },
            }
        }

        println!("Client :: Done poisoning TCPClient");

        let client_poison_Result = client_poison_pill.shutdown();
        

        println!("Client :: Poisoned done");

        45
    });

    (handler_copy, join_handle)
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


