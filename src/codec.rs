
use serde_json;
use bincode;
use bytes;

use serde::Serialize;
use serde::de::DeserializeOwned;

use bytes::BufMut;

use std::str;

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
        if let Some(as_str) = str::from_utf8(bytes).ok() {
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
