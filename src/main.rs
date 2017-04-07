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
use std::net::Shutdown;

use futures::{Future, Stream, Sink};
use futures::sync::oneshot;

use tokio_core::net::{TcpListener, TcpStream};
use tokio_core::reactor::Core;
use tokio_io::io::write_all;
use tokio_io::{AsyncRead, AsyncWrite};
use tokio_io::io::{copy, read};
use tokio_io::codec::{Encoder, Decoder};


use futures::stream::{self};

use bytes::{BytesMut, BufMut};

use std::io::Cursor;


use tokio_io::codec::length_delimited;


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

        // let (reader, writer) = socket.split();
        // let amt = copy(reader, writer);
        // let done = amt.then(move |result| {
        //     match result {
        //         Ok((amt, _, _)) => println!("wrote {} bytes to {}", amt, addr),
        //         Err(e) => println!("error on {}: {}", addr, e),
        //     }

        //     Ok(())
        // });


    
    let srv = socket.incoming().for_each(move |(socket, addr)| {
        println!("got a connection to {:?}", addr);
        // socket = 5;

        // let (writer, mut reader) = socket.split();

        // let x = reader.shutdown();
        // x = 4;
        // let bullshit = socket.clone();
        // socket.shutdown(Shutdown::Both);

        let (sink, stream) = bind_transport(socket).split();

        
        // stream.shutdown();

        let fuckshit = stream.fold(sink, move |mut snk, m| {
            println!("hey mang, I got a message -> {:?}", m);
            // snk.shutdown();
            // snk.close()
            snk.send(m)
        }); // .map(|_|())
        let done = fuckshit.then(move |res| {
            println!("uhhh, what's res fucko -> {:?}", res);
            // Ok(())
            Err(())
        }); // .map_err(|_| ())
 
        handle.spawn(done);

        Ok(())
    });



   
    
    // handle.spawn(srv.map_err(|e| panic!("srv error: {}", e)));

    println!("fall through");

    use std::{thread, time};

    thread::spawn(move || {
        println!("starting client, pre sleep");
        thread::sleep(time::Duration::from_millis(100));
        println!("starting client, post sleep");

        let mut core = Core::new().unwrap();
        let handle = core.handle();
        // some work here
        let client = TcpStream::connect(&addr, &handle);
        let client = core.run(client).unwrap();

        let (client, _) = core.run(write_all(client, frame_for("hi".into()))).unwrap();
        let (client, _) = core.run(write_all(client, frame_for("sup mang".into()))).unwrap();
        let (client, buf, amt) = core.run(read(client, vec![0; 1024])).unwrap();
        client.shutdown(Shutdown::Write).unwrap();
        use std::str;

        let mah_string = str::from_utf8(&buf[..20]).unwrap();
        println!("client done -> {:?}", mah_string);
    });

    // srv = 4;

    let (poison_sender, poison_receiver) = oneshot::channel::<i32>();

    thread::spawn(|| {
        println!("poisin sender about to sleep for 500ms");
        thread::sleep(time::Duration::from_millis(500));
        println!("done sending!");
        poison_sender.send(12).unwrap();
    });

    println!("pre server");
    // core.run(srv).unwrap();

    let without_error = srv.map_err(|_| () );
    core.handle().spawn(without_error);
    println!("post spawn");

    core.run(poison_receiver).unwrap();
    
    println!("post run");






    // handle.spawn(srv.map_err(|e| panic!("srv error: {}", e)));
}

pub fn frame_for(message:String) -> BytesMut {
    let mut frame = BytesMut::with_capacity(32);
    let message_bytes = message.into_bytes();
    frame.put_u32::<BigEndian>(message_bytes.len() as u32);
    frame.put(message_bytes);
    frame
}

