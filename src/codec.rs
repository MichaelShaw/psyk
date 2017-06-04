
use serde_json;
use bincode;
use bytes;

use serde::Serialize;
use serde::de::DeserializeOwned;

use bytes::BufMut;

use std::str;

#[derive(Debug)]
pub enum CodecError {
    CouldntCreateString(str::Utf8Error),
    BinCodeError(bincode::Error), // Box<bincode::ErrorKind>
    JsonError(serde_json::error::Error),
}

// this should be fallible with proper error results ... ... gotta unity various stuff :-/
pub trait Codec<IE, OE> where OE : Serialize, IE : DeserializeOwned {
    fn serialize_outgoing(oe: &OE, bytes: &mut bytes::BytesMut) -> Result<(), CodecError>;
    fn deserialize_incoming(bytes: &bytes::BytesMut) -> Result<IE, CodecError>;
}

pub struct JsonCodec;
impl<IE, OE> Codec<IE, OE> for JsonCodec where OE : Serialize, IE : DeserializeOwned {
    fn serialize_outgoing(oe: &OE, bytes: &mut bytes::BytesMut) -> Result<(), CodecError> {
        match serde_json::to_string(&oe) {
            Ok(string) => {
                bytes.put(string);
                Ok(())
            },
            Err(e) => {
                Err(CodecError::JsonError(e))
            }
        }
    }

    fn deserialize_incoming(bytes: &bytes::BytesMut) -> Result<IE, CodecError> {
        match str::from_utf8(bytes) {
            Ok(as_str) => {
                match serde_json::from_str::<IE>(as_str) {
                    Ok(incoming_event) => Ok(incoming_event),
                    Err(e) => {
                        Err(CodecError::JsonError(e))
                    },
                }
            },
            Err(e) => {
                Err(CodecError::CouldntCreateString(e))
            }
        }
    }
}

pub struct BincodeCodec;
impl<IE, OE> Codec<IE, OE> for BincodeCodec where OE : Serialize, IE : DeserializeOwned {
    fn serialize_outgoing(oe: &OE, bytes: &mut bytes::BytesMut) -> Result<(), CodecError> {
        match bincode::serialize(oe, bincode::Infinite) {
            Ok(bb) => {
                bytes.put(bb);
                Ok(())
            }
            Err(e) => {
                Err(CodecError::BinCodeError(e))
            }
        }
    }

    fn deserialize_incoming(bytes: &bytes::BytesMut) -> Result<IE, CodecError> {
        match bincode::deserialize::<IE>(bytes) {
            Ok(event) => Ok(event),
            Err(e) => Err(CodecError::BinCodeError(e)),
        }
    }
}
