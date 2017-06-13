


use std::net::SocketAddr;

use std;

use {PsykResult, PoisonPill, bind_transport};

// use std::sync::mpsc::Sender;

use serde::{Serialize}; // Deserialize
use serde::de::{DeserializeOwned};

use futures;
use futures::sync::mpsc::{UnboundedSender};
use futures::sync::oneshot;
use futures::{Stream, Sink, Future};

// use tokio_io::io;
use std::fmt::Debug;


use tokio_core::net::{TcpListener};
use tokio_core::reactor::Core;

use std::thread;

use bytes::{BytesMut};

use super::codec::AsymmetricCodec;


// we could in theory hand one of these directly to the client ...
#[derive(Clone)]
pub struct ServerEventHandler<SIE, SOE> { // this is a "logical" handle for the server loop
    pub sender: std::sync::mpsc::Sender<ServerInboundEvent<SIE, SOE>>, // how the tcp server sends event to the server loop
}


#[derive(Debug, Clone)]
pub enum ServerInboundEvent<SIE, SOE> {
    ClientConnected { address : SocketAddr, client_sender : UnboundedSender<SOE> },
    ClientMessage { address: SocketAddr, event: SIE },
    ClientDisconnected { address : SocketAddr },
    FailureToBind { address : SocketAddr }, // last 2 events could be combined in some form of "TCPServer finished with Result ...."
    ServerFinished { address: SocketAddr },
}


pub fn run_server<SIE, SOE, C>(server_handler:ServerEventHandler<SIE, SOE>, bind_address: SocketAddr) -> PsykResult<PoisonPill>
     where SIE : DeserializeOwned + Send + Clone + Debug + 'static, SOE : Serialize + Send + Clone + Debug + 'static, C: AsymmetricCodec<SIE, SOE> { // spawns a server and returns a poison pill handle ... that can be used to terminate the server
    let (poison_sender, poison_receiver) = oneshot::channel();


    // what do we do if we can't bind :-/ ... send a failure to bind event
    
    let join_handle = thread::spawn(move || {
        println!("TCPServer :: starting");
        create_server::<SIE, SOE, C>(server_handler, bind_address, poison_receiver);
        println!("TCPServer :: finished");
        12
    });

    Ok(PoisonPill {
        sender: poison_sender,
        join_handle: join_handle,
    })
}


pub fn create_server<SIE, SOE, C>(server_handler:ServerEventHandler<SIE, SOE>, bind_address: SocketAddr, poison_receiver: oneshot::Receiver<u32>) 
    where SIE : DeserializeOwned + 'static + Clone + Debug, SOE : Serialize + 'static + Clone + Debug, C: AsymmetricCodec<SIE, SOE> {
    let mut core = Core::new().expect("TCPSERVER A NEW CORE"); // io result

    let handle = core.handle();
    
    let socket = TcpListener::bind(&bind_address, &handle).expect("TCPSERVER BIND");

    let server_handler_copy = server_handler.clone();

    let srv = socket.incoming().for_each(move |(socket, addr)| {
        println!("TCPServer :: got a connection to {:?}", addr);

        let (client_send, client_receive) = futures::sync::mpsc::unbounded();
        let other_handle = server_handler.clone();
        let hhrrrm = server_handler.clone();

        let (sink, stream) = bind_transport(socket).split();

        // use the raw send
        hhrrrm.sender.send(ServerInboundEvent::ClientConnected { address : addr, client_sender : client_send }).expect("TCPSERVER SEND CLIENTCONNECTED");
        
        let socket_reader = stream.for_each(move |m| {
            println!("TCPServer :: hey mang, I got a message -> {:?}", m);

            match C::deserialize_incoming(&m) {
                Ok(ie) => {
                    println!("TCPServer :: received incoming message -> {:?}", ie);
                    hhrrrm.sender.send(ServerInboundEvent::ClientMessage { address : addr, event : ie }).expect("TCPSERVER SEND CLIENTMESSAGE");
                },
                Err(e) => println!("TCPServer :: couldnt deserialize incoming message -> {:?}", e),
            }

            Ok(())
        }); 

        let socket_writer = client_receive.fold(sink, |sink, msg| {
            println!("TCPServer :: writing an outbound event to the client -> {:?}", msg);
            let mut some_bytes : BytesMut = BytesMut::new();
            match C::serialize_outgoing(&msg, &mut some_bytes) {
                Ok(()) => (),
                Err(e) => println!("TCPServer :: couldnt serialize event -> {:?}", e),
            }
            let amt = sink.send(some_bytes); // should only do this on happy path
            amt.map_err(|_| ())
        });

        let socket_reader = socket_reader.map_err(|_| ());
        let connection = socket_reader.map(|_| ()).select(socket_writer.map(|_| ()));
        handle.spawn(connection.then(move |_| {
            println!("TCPServer :: Connection {} closed.", addr);
            &other_handle.sender.send(ServerInboundEvent::ClientDisconnected { address : addr }).expect("TCPSERVER SEND CLIENTDISCONNECT");
            Ok(())
        }));
        
        Ok(())
    });

    let without_error = srv.map_err(|_| () );

    core.handle().spawn(without_error);

    core.run(poison_receiver).expect("TCPSERVER RUN");

    server_handler_copy.sender.send(ServerInboundEvent::ServerFinished { address : bind_address }).expect("TCPSERVER SEND SERVERFINISHED");
}