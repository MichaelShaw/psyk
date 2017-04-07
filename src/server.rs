

use super::*;

use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

use futures::sync::mpsc::{UnboundedSender, UnboundedReceiver};
use futures::sync::oneshot;
use futures::{Stream, Sink, Future};

use tokio_io::io;
use tokio_io::{AsyncRead, AsyncWrite};
use tokio_io::codec::length_delimited;

use tokio_core::net::{TcpListener, TcpStream};
use tokio_core::reactor::Core;

use std::thread;

use bytes::{BytesMut, BufMut, Bytes};


// we could in theory hand one of these directly to the client ...
#[derive(Clone)]
pub struct ServerHandle<SE, CE> {
    pub sender: UnboundedSender<ServerEvent<SE, CE>>, // how the tcp server sends event to the server loop
}

pub struct ServerPoisonPill {
    pub sender : oneshot::Sender<u32>,
    pub join_handle : thread::JoinHandle<u32>,
}

#[derive(Debug, Clone)]
pub enum ServerEvent<SE, CE> {
    ClientConnected { address : SocketAddr, client_sender : UnboundedSender<CE> },
    ClientMessage { address: SocketAddr, event: SE },
    ClientDisconnected { address : SocketAddr },
}

fn bind_transport<T: AsyncRead + AsyncWrite>(io: T) -> length_delimited::Framed<T> {
    length_delimited::Framed::new(io) // by default a big endian u32 at the start
}

pub fn run_server<SE, CE>(server_handle:ServerHandle<SE, CE>, bind_address: SocketAddr) -> PsykResult<ServerPoisonPill> where SE : Deserialize + Send + Clone + 'static, CE : Serialize + Send + Clone + 'static { // spawns a server and returns a poison pill handle ... that can be used to terminate the server
    let (poison_sender, poison_receiver) = oneshot::channel();
    
    let join_handle = thread::spawn(move || {
        create_server(server_handle, bind_address, poison_receiver);
        12
    });

    Ok(ServerPoisonPill {
        sender: poison_sender,
        join_handle: join_handle,
    })
}

pub fn create_server<SE, CE>(server_handle:ServerHandle<SE, CE>, bind_address: SocketAddr, poison_receiver: oneshot::Receiver<u32>) where SE : Deserialize + 'static + Clone, CE : Serialize + 'static + Clone {
    let mut core = Core::new().unwrap();

    let handle = core.handle();
    
    let socket = TcpListener::bind(&bind_address, &handle).unwrap();

    let srv = socket.incoming().for_each(move |(socket, addr)| {
        println!("got a connection to {:?}", addr);

        let (client_send, client_receive) = futures::sync::mpsc::unbounded();
        let other_handle = server_handle.clone();
        let hhrrrm = server_handle.clone();

        let (sink, stream) = bind_transport(socket).split();

        // use the raw send
        UnboundedSender::<ServerEvent<SE, CE>>::send(&hhrrrm.sender, ServerEvent::ClientConnected { address : addr, client_sender : client_send }).unwrap();

        let socket_reader = stream.for_each(move |m| {
            println!("hey mang, I got a message -> {:?}", m);

            if let Some(as_str) = std::str::from_utf8(&m).ok() {
                 match serde_json::from_str::<SE>(as_str) {
                    Ok(event) => UnboundedSender::<ServerEvent<SE, CE>>::send(&hhrrrm.sender, ServerEvent::ClientMessage { address : addr, event : event }).unwrap(),
                    Err(e) => println!("couldnt deserialize event ... error -> {:?} string -> {} ", e, as_str),
                 }
            } else {
                println!("couldnt create utf8 string from frame!!");
            }

            Ok(())
        }); 

        let socket_writer = client_receive.fold(sink, |sink, msg| {
            println!("writing a client event!");
            let msg = serde_json::to_string(&msg).unwrap();
            let some_bytes : BytesMut = BytesMut::from(msg);
            let amt = sink.send(some_bytes);
            amt.map_err(|_| ())
        });

        let socket_reader = socket_reader.map_err(|_| ());
        let connection = socket_reader.map(|_| ()).select(socket_writer.map(|_| ()));
        handle.spawn(connection.then(move |_| {
            // connections.borrow_mut().remove(&addr);
            println!("Connection {} closed.", addr);
            UnboundedSender::<ServerEvent<SE, CE>>::send(&other_handle.sender, ServerEvent::ClientDisconnected { address : addr }).unwrap();
            Ok(())
        }));
        

        Ok(())
    });

    let without_error = srv.map_err(|_| () );

    core.handle().spawn(without_error);

    core.run(poison_receiver).unwrap();
}