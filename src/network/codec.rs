
use bytes::{BytesMut};

use serde::Serialize;
use serde::de::DeserializeOwned;

use aphid::codec::*;


pub trait AsymmetricCodec<IE, OE> where OE : Serialize, IE : DeserializeOwned { // for client <-> server use
    fn serialize_outgoing(oe: &OE, bytes: &mut BytesMut) -> Result<(), CodecError>;
    fn deserialize_incoming(bytes: &[u8]) -> Result<IE, CodecError>;
}

impl<IE, OE> AsymmetricCodec<IE, OE> for JsonCodec where OE : Serialize, IE : DeserializeOwned {
    fn serialize_outgoing(oe: &OE, bytes: &mut BytesMut) -> Result<(), CodecError> {
        serialize_json_bytes(oe, bytes)
    }

    fn deserialize_incoming(bytes: &[u8]) -> Result<IE, CodecError> {
        deserialize_json(bytes)
    }
}


impl<IE, OE> AsymmetricCodec<IE, OE> for BincodeCodec where OE : Serialize, IE : DeserializeOwned {
    fn serialize_outgoing(oe: &OE, bytes: &mut BytesMut) -> Result<(), CodecError> {
        serialize_bincode_bytes(oe, bytes)
    }

    fn deserialize_incoming(bytes: &[u8]) -> Result<IE, CodecError> {
        deserialize_bincode(bytes)
    }
}
