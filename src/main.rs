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

    let server = server_handle.clone();



    let poison_pill = psyk::server::run_server(server_handle, addr).unwrap();

    println!("alright, server started, sending the damn thing");

    server.sender.send(psyk::server::ServerInboundEvent::ClientMessage { address: addr, event : MathServerEvent::Get }).unwrap();

    let res = poison_pill.shutdown();

    println!("ok we even shutdown -> {:?}", res);


    let (client_tx, client_rx) = mpsc::channel();

    let (poison_pill, server_tx) = psyk::client::run_client::<MathServerEvent, MathClientEvent>(client_tx, addr).unwrap();



}

fn spawn_math_client(client_rx : mpsc::Receiver<psyk::client::ClientInboundEvent<MathClientEvent>>) {

    let join_handle = thread::spawn(move || {

        45
    });

}

fn spawn_math_server() -> (ServerHandle<MathServerEvent, MathClientEvent>, JoinHandle<u32>) {
    use std::collections::HashMap;
    
    let (sender, receiver) = mpsc::channel();

    let join_handle = thread::spawn(move || {
        let mut val = 0_u32;

        println!("math server entering main loop");

        let mut clients = HashMap::new();

        use psyk::server::ServerInboundEvent::*;
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


