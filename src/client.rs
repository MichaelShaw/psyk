
use tokio_core::reactor::Core;
use tokio_core::net::{TcpStream};

// use futures::sync::mpsc::Unvou

#[derive(Clone)]
pub struct ClientHandle { // <SE, CE>
    // pub sender: std::sync::mpsc::Receiver<Client<SE, CE>>, // how the tcp server sends event to the server loop
}

#[derive(Debug, Clone)]
pub enum ClientEvent<CE> {
    ServerConnected,
    ServerMessage { event: CE },
    ServerDisconnected,
}

fn run_client() {
	use std::{thread, time};

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
}


// pub fn frame_for(message:String) -> BytesMut {
//     let mut frame = BytesMut::with_capacity(32);
//     let message_bytes = message.into_bytes();
//     frame.put_u32::<BigEndian>(message_bytes.len() as u32);
//     frame.put(message_bytes);
//     frame
// }

