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

type MyCodec = psyk::BincodeCodec;



fn main() {
    let addr = env::args().nth(1).unwrap_or("127.0.0.1:8080".to_string());
    let addr : SocketAddr = addr.parse().expect("A valid server bind address");

    let (server_event_handler, server_join_handle) = spawn_math_server();

    println!("Main :: Server Event Handler Started");
    let server_poison_pill = psyk::server::run_server::<_, _, MyCodec>(server_event_handler.clone(), addr).expect("A SERVER POISON PILL");

    println!("Main :: Server TCPListener Started");
    thread::sleep(time::Duration::from_millis(100));

    let client_count = 1;
    let client_handles : Vec<_> = (0..client_count).map(|n|{ 
        spawn_math_client(addr, n)
    }).collect();

    // let (client_event_handler, client_join_handler) = ;
    println!("Main :: Client Event Handler Started");

    thread::sleep(time::Duration::from_millis(100));
    let res = server_poison_pill.shutdown();

    println!("Main :: Waiting for Client to shutdown");

    let mut n = 0;
    for (_, client_join_handle) in client_handles {
        let client_res = client_join_handle.join();
        println!("Main :: joined client {}", n);
        n += 1;
    }

    println!("Main :: Clients shutdown, posioning server");

    

    println!("Main :: ok we even shutdown -> {:?}", res);
}

fn spawn_math_client(server_address: SocketAddr, n: u32) -> (ClientEventHandler<MathToClientEvent, MathToServerEvent>, JoinHandle<u32>) {
    use psyk::client::ClientInboundEvent::*;

    let (sender, receiver) = mpsc::channel();

    // state for server

    let client_event_handler = ClientEventHandler {
        sender : sender,
    };

    let handler_copy = client_event_handler.clone();


    let join_handle = thread::spawn(move || {
        println!("Client {} :: connecting to {:?}", n, server_address);
        // attempt to connect to server
        let client_poison_pill = psyk::client::run_client::<_, _, MyCodec>(client_event_handler.clone(), server_address).expect("MATH CLIENT EXPECTS A CLIENT POISON PILL");

        let mut to_server : Option<psyk::client::ChannelToServer<MathToServerEvent>> = None;

        loop {
            match receiver.recv() {
                Ok(event) => {
                    match event {
                        FailedToConnect { address } => {
                            println!("Client {} :: failed to connect to {:?}", n, address);
                            break;
                        },
                        ServerConnected { address, channel_to_server } => {
                            println!("Client {} :: connected to server @ {:?}", n, address);
                            // psyk::client::ClientOutboundEvent::SendMessage { event: MathToServerEvent::Get }
                            channel_to_server.sender.send(MathToServerEvent::Add(n));

                            println!("Client {} :: Sending get", n);
                            to_server = Some(channel_to_server)
                            
                        }, // that is NOT good enough ..
                        ServerMessage { address, event } => {
                            println!("Client {} :: received event {:?} from server @ {:?}", n, event, address);
                            // break;
                        },
                        ServerDisconnected { address } => {
                            println!("Client {} :: server disconnected @ {:?}", n, address);
                            break;
                        },
                        ClientFinished { address } => {
                            println!("Client {} :: finished event", n);
                            break;
                        }
                    }

                },
                Err(e) => {
                    println!("Client {} :: problem receiving event :-( {:?}", n, e);
                    break;
                },
            }
        }

        println!("Client {} :: Done poisoning TCPClient", n);

        let client_poison_result = client_poison_pill.shutdown();
        

        println!("Client {} :: Poisoned done", n);

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
                        ServerFinished { address }=> {
                            println!("Server :: server process @ {:?} finished", address);
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
                                sender.send(MathToClientEvent::Val(val)).expect("SERVER EXPECTS TO BE ABLE TO SEND EVENT TO TCPSERVER");
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


