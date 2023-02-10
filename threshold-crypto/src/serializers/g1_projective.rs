use bls12_381::{G1Affine, G1Projective};
use group::Curve;
use serde::{
    de::{Error as DeserializeError, SeqAccess, Visitor},
    ser::SerializeTuple,
    Deserializer, Serializer,
};
use std::fmt;

const COMPRESSED_SIZE: usize = 48;
const DESERIALIZE_ERROR_MSG: &'static str = "deserialized bytes don't encode a group element";

pub fn serialize<S>(g1: &G1Projective, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut serialize_tuple = s.serialize_tuple(COMPRESSED_SIZE)?;
    let bytes = g1.to_affine().to_compressed();
    for byte in &bytes {
        serialize_tuple.serialize_element(byte)?;
    }
    serialize_tuple.end()
}

struct TupleVisitor;

impl<'de> Visitor<'de> for TupleVisitor {
    type Value = G1Projective;
    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "a tuple of size {}", COMPRESSED_SIZE)
    }
    #[inline]
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let mut compressed: [u8; 48] = [0; 48];
        for (i, byte) in compressed.as_mut().iter_mut().enumerate() {
            let len_err = || DeserializeError::invalid_length(i, &self);
            *byte = seq.next_element()?.ok_or_else(len_err)?;
        }
        let g1_affine = G1Affine::from_compressed(&compressed);
        if g1_affine.is_none().into() {
            Err(DeserializeError::custom(DESERIALIZE_ERROR_MSG))
        } else {
            let g1_affine = g1_affine.unwrap();
            Ok(g1_affine.into())
        }
    }
}

pub fn deserialize<'de, D>(d: D) -> Result<G1Projective, D::Error>
where
    D: Deserializer<'de>,
{
    d.deserialize_tuple(COMPRESSED_SIZE, TupleVisitor)
}
