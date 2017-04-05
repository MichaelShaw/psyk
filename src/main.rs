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

use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::iter;
use std::env;
use std::io::{Error, ErrorKind, BufReader};
use std::io;

use futures::{Future, Stream, Sink};


use tokio_core::net::{TcpListener, TcpStream};
use tokio_core::reactor::Core;
use tokio_io::io::write_all;
use tokio_io::AsyncRead;
use tokio_io::io::{copy, read};
use tokio_io::codec::{Encoder, Decoder};


use futures::stream::{self};

use bytes::{BytesMut, BufMut};

use std::io::Cursor;


enum Message {
    Server(ServerMessage),
    Client(ClientMessage),
}

enum ClientMessage {
    Add(u32),
    Subrtact(u32),
    Multiply(u32),
    Divide(u32),
    Get(),
}

enum ServerMessage {
    Ok(u32),
    Rejected(),
}

use tokio_io::codec::length_delimited;
use tokio_io::{AsyncWrite};

fn bind_transport<T: AsyncRead + AsyncWrite>(io: T) -> length_delimited::Framed<T> {
    length_delimited::Framed::new(io) // by default a big endian u32 at the start
}

use futures::sync::mpsc::UnboundedSender;

use bytes::{BigEndian};

fn main() {
    let addr = env::args().nth(1).unwrap_or("127.0.0.1:8080".to_string());
    let addr = addr.parse().unwrap();

    let mut core = Core::new().unwrap();
    let handle = core.handle();
    
    let socket = TcpListener::bind(&addr, &handle).unwrap();
    println!("Listening on: {}", addr);

    let connections : Rc<RefCell<HashMap<std::net::SocketAddr, UnboundedSender<String>>>> = Rc::new(RefCell::new(HashMap::new()));

    
    let srv = socket.incoming().for_each(move |(socket, addr)| {
        println!("got a connection to {:?}", addr);
        // let transport = bind_transport(socket);
        // let echo = transport.for_each(|buf| {
        //     println!("we got buf -> {:?}", buf);
        //     Ok(())
        // });

        let (sink, stream) = bind_transport(socket).split();

        let fuckshit = stream.fold(sink, move |snk, m| {
            snk.send(m)
        });
        // let echo = stream.for_each(move |buf| {
        //     println!("we got buf -> {:?}", buf);
        //     println!("what is sink -> {:?}", sink);
        //     // sink.send(buf)
        //     Ok(())
        // });



        let done = fuckshit.and_then(|res| {
            println!("ok, good or bad, we done either way -> {:?}", res);
            Ok(())
        }).map_err(|_| ());
 
        handle.spawn(done);

        Ok(())
    });
   
    let handle = core.handle();
    handle.spawn(srv.map_err(|e| panic!("srv error: {}", e)));

    println!("fall through");
    // core.run(srv).unwrap();
    let client = TcpStream::connect(&addr, &handle);
    let client = core.run(client).unwrap();

    let (client, _) = core.run(write_all(client, frame_for("hi".into()))).unwrap();
    let (client, _) = core.run(write_all(client, frame_for("sup mang".into()))).unwrap();
    let (client, buf, amt) = core.run(read(client, vec![0; 1024])).unwrap();

    use std::str;

    let mah_string = str::from_utf8(&buf[..20]).unwrap();

    println!("mah string -> {:?}", mah_string);

    // handle.spawn(srv.map_err(|e| panic!("srv error: {}", e)));
}

pub fn frame_for(message:String) -> BytesMut {
    let mut frame = BytesMut::with_capacity(32);
    let message_bytes = message.into_bytes();
    frame.put_u32::<BigEndian>(message_bytes.len() as u32);
    frame.put(message_bytes);
    frame
}

