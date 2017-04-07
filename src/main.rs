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

use psyk::server::ServerHandle;

use std::thread;
use std::thread::JoinHandle;

use std::sync::mpsc;

#[derive(Serialize, Deserialize, Clone, Debug)]
enum MathServerEvent {
    Get,
    Add(u32),
    Sub(u32),
    Mul(u32),
    Div(u32),
}

#[derive(Serialize, Deserialize, Clone, Debug)] 
enum MathClientEvent {
    Val(u32),
}

fn main() {
    let addr = env::args().nth(1).unwrap_or("127.0.0.1:8080".to_string());
    let addr : SocketAddr = addr.parse().unwrap();

    let (server_handle, join_handle) = spawn_math_server();

    let bullshit = server_handle.clone();


    let poison_pill = psyk::server::run_server(server_handle, addr).unwrap();

    println!("alright, server started, sending the damn thing");

    bullshit.sender.send(psyk::server::ServerEvent::ClientMessage { address: addr, event : MathServerEvent::Get }).unwrap();


    let res = poison_pill.shutdown();

    println!("ok we even shutdown -> {:?}", res);





// UnboundedSender::<ServerEvent<SE, CE>>::send(&hhrrrm.sender, ServerEvent::ClientConnected { address : addr, client_sender : client_send }).unwrap();




    // handle.spawn(srv.map_err(|e| panic!("srv error: {}", e)));
}

fn spawn_math_server() -> (ServerHandle<MathServerEvent, MathClientEvent>, JoinHandle<u32>) {
    use std::collections::HashMap;
    
    let (sender, receiver) = mpsc::channel();

    let join_handle = thread::spawn(move || {
        let mut val = 0_u32;

        println!("math server entering main loop");

        let mut clients = HashMap::new();

        use psyk::server::ServerEvent::*;
        use MathServerEvent::*;

        loop {
            match receiver.recv() {
                Ok(event) => {
                    match event {
                        ClientConnected { address, client_sender } => {
                            println!("client connected :D");
                            clients.insert(address, client_sender);
                        },
                        ClientMessage { address, event } => {
                            println!("client message -> {:?}", event);
                            match event {
                                Get => (),
                                Add(n) => val += n,
                                Sub(n) => val -= n,
                                Mul(n) => val *= n,
                                Div(n) => val /= n,
                            }

                            if let Some(sender) = clients.get(&address) {
                                println!("sending current value {} to client {:?}", val, address);
                                sender.send(MathClientEvent::Val(val)).unwrap();
                            } else {
                                println!("we're not aware of client {:?}", address);
                            }
                        },
                        ClientDisconnected { address } => {
                            println!("client disconnected");
                            clients.remove(&address);
                        },
                    }
                },
                Err(e) => println!("math server problem receiving event :-( {:?}", e),
            }
        }
        //     println!("math server got message -> {:?}", event);
  
       

        println!("math server done");


        32
    });

    (ServerHandle {
        sender : sender,
    }, join_handle)
}


