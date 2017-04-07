

extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;


extern crate bytes;

extern crate futures;
#[macro_use]
extern crate tokio_core;
extern crate tokio_io;

use tokio_io::codec::length_delimited;
use tokio_io::{AsyncRead, AsyncWrite};

use futures::sync::oneshot;
use std::thread;

pub mod server;
pub mod client;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}

use std::io;

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
