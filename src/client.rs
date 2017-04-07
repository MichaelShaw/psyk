use super::*;

use tokio_core::reactor::Core;
use tokio_core::net::{TcpStream};

use serde::{Deserialize, Serialize};

use std::net::SocketAddr;

use futures::sync::mpsc::{UnboundedSender, UnboundedReceiver};
use futures::sync::oneshot;
use futures::{Future, Stream, Sink};

use std::sync::mpsc::{Receiver, Sender};
use std::thread;

use bytes::{BytesMut};

// connected to a single client
pub struct ChannelToServer<SE> { // <SE, CE>
    pub sender: UnboundedSender<SE>, // how the tcp server sends event to the server loop
}

#[derive(Debug, Clone)]
pub enum ClientEvent<CE> {
    ServerConnected,
    ServerMessage { event: CE },
    ServerDisconnected,
}

pub fn run_client<SE, CE>(client_sender: Sender<ClientEvent<CE>>, server_address:SocketAddr) -> PsykResult<(PoisonPill, UnboundedSender<SE>)> where SE : Serialize + Send + Clone + 'static, CE : Deserialize + Send + Clone + 'static {
    let (poison_sender, poison_receiver) = oneshot::channel();

    let (server_tx, server_rx) = futures::sync::mpsc::unbounded::<SE>();

    let join_handle = thread::spawn(move || {
        println!("tcp server starting");
        // create_server(server_handle, bind_address, poison_receiver);

        connect_client_to(client_sender, server_rx, server_address, poison_receiver);
        println!("tcp server over");
        12
    });

    Ok((PoisonPill {
        sender: poison_sender,
        join_handle: join_handle,
    }, server_tx))
}

fn connect_client_to<SE, CE>(client_sender: Sender<ClientEvent<CE>>, server_rx:UnboundedReceiver<SE>, server_address:SocketAddr, poison_receiver: oneshot::Receiver<u32>) where SE : Serialize + Send + Clone + 'static, CE : Deserialize + Send + Clone + 'static {
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let tcp = TcpStream::connect(&server_address, &handle);

    


    let client = tcp.and_then(move |stream| {
        let (sink, stream) = bind_transport(stream).split();

        client_sender.send(ClientEvent::ServerConnected).unwrap();

        let client_copy = client_sender.clone();

        let socket_reader = stream.for_each(move |m| {
            if let Some(as_str) = std::str::from_utf8(&m).ok() {
                 match serde_json::from_str::<CE>(as_str) {
                    Ok(event) => client_sender.send(ClientEvent::ServerMessage { event : event }).unwrap(),
                    Err(e) => println!("couldnt deserialize event ... error -> {:?} string -> {} ", e, as_str),
                 }
            } else {
                println!("couldnt create utf8 string from frame!!");
            }

            Ok(())
        });

        let socket_writer = server_rx.fold(sink, |sink, msg| {
            println!("writing an event for the server!");
            let msg = serde_json::to_string(&msg).unwrap();
            let some_bytes : BytesMut = BytesMut::from(msg);
            let amt = sink.send(some_bytes);
            amt.map_err(|_| ())
        });


        let socket_reader = socket_reader.map_err(|_| ());
        let connection = socket_reader.map(|_| ()).select(socket_writer.map(|_| ()));
        handle.spawn(connection.then(move |_| {
            // connections.borrow_mut().remove(&addr);
            println!("Connection {} close to server.", server_address);
            client_copy.send(ClientEvent::ServerDisconnected).unwrap();
            Ok(())
        }));



        Ok(())
    });

    let without_error = client.map_err(|_| () );

    core.handle().spawn(without_error);

    core.run(poison_receiver).unwrap();
}


// fn run_client() {
    // use std::{thread, time};

    // thread::spawn(move || {
    //     println!("starting client, pre sleep");
    //     thread::sleep(time::Duration::from_millis(100));
    //     println!("starting client, post sleep");

    //     let mut core = Core::new().unwrap();
    //     let handle = core.handle();
    //     // some work here
    //     let client = TcpStream::connect(&addr, &handle);
    //     let client = core.run(client).unwrap();

    //     let (client, _) = core.run(write_all(client, frame_for("hi".into()))).unwrap();
    //     let (client, _) = core.run(write_all(client, frame_for("sup mang".into()))).unwrap();
    //     let (client, buf, amt) = core.run(read(client, vec![0; 1024])).unwrap();
    //     client.shutdown(Shutdown::Write).unwrap();
    //     use std::str;

    //     let mah_string = str::from_utf8(&buf[..20]).unwrap();
    //     println!("client done -> {:?}", mah_string);
    // });
// }


// pub fn frame_for(message:String) -> BytesMut {
//     let mut frame = BytesMut::with_capacity(32);
//     let message_bytes = message.into_bytes();
//     frame.put_u32::<BigEndian>(message_bytes.len() as u32);
//     frame.put(message_bytes);
//     frame
// }

