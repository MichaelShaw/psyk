

extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;


extern crate bytes;

extern crate futures;
#[macro_use]
extern crate tokio_core;
extern crate tokio_io;


pub mod server;
pub mod client;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}

use std::io;

pub enum PsykError {
	IO(io::Error)
}

impl From<io::Error> for PsykError {
    fn from(err: io::Error) -> Self {
        PsykError::IO(err)
    }
}


pub type PsykResult<T> = Result<T, PsykError>;