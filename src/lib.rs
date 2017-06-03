

extern crate serde;

// #[macro_use]
// extern crate serde_derive;

extern crate bytes;

extern crate serde_json;
extern crate bincode;

extern crate futures;
// #[macro_use]
extern crate tokio_core;
extern crate tokio_io;

use tokio_io::codec::length_delimited;
use tokio_io::{AsyncRead, AsyncWrite};

use futures::sync::oneshot;
use std::thread;

pub mod server;
pub mod client;

use std::io;

use serde::Serialize;
use serde::de::DeserializeOwned;

use bytes::BufMut;

#[derive(Debug)]
pub enum PsykError {
	IO(io::Error)
}

impl From<io::Error> for PsykError {
    fn from(err: io::Error) -> Self {
        PsykError::IO(err)
    }
}


pub type PsykResult<T> = Result<T, PsykError>;

pub fn bind_transport<T: AsyncRead + AsyncWrite>(io: T) -> length_delimited::Framed<T> {
    length_delimited::Framed::new(io) // by default a big endian u32 at the start
}


pub struct PoisonPill {
    pub sender : oneshot::Sender<u32>,
    pub join_handle : thread::JoinHandle<u32>,
}

impl PoisonPill {
    pub fn shutdown(self) -> std::result::Result<u32, std::boxed::Box<std::any::Any + std::marker::Send>> {
        self.sender.send(99).unwrap();
        self.join_handle.join()
    }
}

// this should be fallible with proper error results ... ... gotta unity various stuff :-/
pub trait Codec<IE, OE> where OE : Serialize, IE : DeserializeOwned {
    fn serialize_outgoing(oe: &OE, bytes: &mut bytes::BytesMut);
    fn deserialize_incoming(bytes: &bytes::BytesMut) -> Option<IE>;
}

pub struct JsonCodec;
impl<IE, OE> Codec<IE, OE> for JsonCodec where OE : Serialize, IE : DeserializeOwned {
    fn serialize_outgoing(oe: &OE, bytes: &mut bytes::BytesMut) {
        let string = serde_json::to_string(&oe).expect("JsonCodec :: SERIALIZE THAT OUTGOING");
        bytes.put(string);
    }

    fn deserialize_incoming(bytes: &bytes::BytesMut) -> Option<IE> {
        if let Some(as_str) = std::str::from_utf8(bytes).ok() {
            match serde_json::from_str::<IE>(as_str) {
                Ok(incoming_event) => Some(incoming_event),
                Err(e) => {
                    println!("JsonCodec :: couldnt deserialize event ... error -> {:?} string -> {} ", e, as_str);
                    None
                },
            }
        } else {
            None
        }
    }
}

pub struct BincodeCodec;
impl<IE, OE> Codec<IE, OE> for BincodeCodec where OE : Serialize, IE : DeserializeOwned {
    fn serialize_outgoing(oe: &OE, bytes: &mut bytes::BytesMut) {
         let bb = bincode::serialize(oe, bincode::Infinite).expect("BincodeCodec :: SERIALIZE THAT OUTGOING");
         bytes.put(bb);
    }

    fn deserialize_incoming(bytes: &bytes::BytesMut) -> Option<IE> {
        if let Some(event) = bincode::deserialize::<IE>(bytes).ok() {
            Some(event)
        } else {
            None
        }
    }
}
