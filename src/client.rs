use super::*;

use tokio_core::reactor::Core;
use tokio_core::net::{TcpStream};

use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;


use std::fmt::Debug;
use std::net::SocketAddr;

use futures::sync::mpsc::{UnboundedSender, UnboundedReceiver};
use futures::sync::oneshot;
use futures::{Future, Stream, Sink};

use std::sync::mpsc::{Receiver, Sender};
use std::thread;

use bytes::{BytesMut};

#[derive(Clone)]
pub struct ClientEventHandler<CIE, COE> { // this is a "logical" handle for the server loop
    pub sender: Sender<ClientInboundEvent<CIE, COE>>, // how the tcp server sends event to the server loop
}
// connected to a single client
#[derive(Debug, Clone)]
pub struct ChannelToServer<COE> { // <SE, CE>
    pub sender: UnboundedSender<COE>, 
}

#[derive(Debug, Clone)]
pub enum ClientOutboundEvent<COE> {
    SendMessage { event: COE },
    Shutdown,
}

#[derive(Debug, Clone)]
pub enum ClientInboundEvent<CIE, COE> {
    // failed to connect in the first place?
    FailedToConnect { address: SocketAddr },
    ServerConnected { address: SocketAddr, channel_to_server: ChannelToServer<COE> }, // that is NOT good enough ..
    ServerMessage { address: SocketAddr, event: CIE },
    ServerDisconnected { address: SocketAddr },
    ClientFinished { address:SocketAddr }, // unsure of if we should have this one
}

pub fn run_client<CIE, COE>(client_handler: ClientEventHandler<CIE, COE>, server_address:SocketAddr) -> PsykResult<PoisonPill> 
        where CIE : DeserializeOwned + Send + Clone + Debug + 'static, COE : Serialize + Send + Clone + Debug + 'static {
    let (poison_sender, poison_receiver) = oneshot::channel();

    let join_handle = thread::spawn(move || {
        println!("TCPClient :: starting");
        // create_server(server_handle, bind_address, poison_receiver);

        connect_client_to(client_handler, server_address, poison_receiver);
        println!("TCPClient :: finished");
        12
    });

    Ok(PoisonPill {
        sender: poison_sender,
        join_handle: join_handle,
    })
}

fn connect_client_to<CIE, COE>(client_handler: ClientEventHandler<CIE, COE>, server_address:SocketAddr, poison_receiver: oneshot::Receiver<u32>) 
        where CIE : DeserializeOwned + Send + Clone + Debug + 'static, COE : Serialize + Send + Clone + Debug + 'static {
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let tcp = TcpStream::connect(&server_address, &handle);

    let client_handler_copy = client_handler.clone();

    let client = tcp.and_then(move |stream| {
        let (sink, stream) = bind_transport(stream).split();

        let client_copy = client_handler.clone();

        let (to_server_tx, to_server_rx) = futures::sync::mpsc::unbounded::<COE>();
        let channel_to_server = ChannelToServer { sender: to_server_tx };
        client_copy.sender.send(ClientInboundEvent::ServerConnected { address: server_address, channel_to_server: channel_to_server }).unwrap();

        let socket_reader = stream.for_each(move |m| {
            if let Some(as_str) = std::str::from_utf8(&m).ok() {
                 match serde_json::from_str::<CIE>(as_str) {
                    Ok(event) => {
                        println!("TCPClient :: received event {:?}", event);
                        client_handler.sender.send(ClientInboundEvent::ServerMessage { address: server_address, event : event }).unwrap();
                    },
                    Err(e) => println!("TCPClient :: couldnt deserialize event ... error -> {:?} string -> {} ", e, as_str),
                 }
            } else {
                println!("TCPClient :: couldnt create utf8 string from frame!!");
            }

            Ok(())
        });

        let socket_writer = to_server_rx.fold(sink, |sink, msg| {
            println!("TCPClient :: writing an outbound event for the server!");
            let msg = serde_json::to_string(&msg).unwrap();
            let some_bytes : BytesMut = BytesMut::from(msg);
            let amt = sink.send(some_bytes);
            amt.map_err(|_| ())
        });


        let socket_reader = socket_reader.map_err(|_| ());
        let connection = socket_reader.map(|_| ()).select(socket_writer.map(|_| ()));
        handle.spawn(connection.then(move |_| {
            // connections.borrow_mut().remove(&addr);
            println!("TcpClient :: Connection {} close to server.", server_address);
            client_copy.sender.send(ClientInboundEvent::ServerDisconnected { address: server_address } ).unwrap();
            Ok(())
        }));

        Ok(())
    });

    let without_error = client.map_err(|_| () );

    core.handle().spawn(without_error);

    core.run(poison_receiver).unwrap();

    client_handler_copy.sender.send(ClientInboundEvent::ClientFinished { address: server_address });
}
