use bls12_381::{G2Affine, G2Projective};
use group::Curve;
use serde::{
    de::{Error as DeserializeError, SeqAccess, Visitor},
    ser::SerializeTuple,
    Deserializer, Serializer,
};
use std::fmt;

const COMPRESSED_SIZE: usize = 96;
const DESERIALIZE_ERROR_MSG: &'static str = "deserialized bytes don't encode a group element";

pub fn serialize<S>(g2: &G2Projective, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut serialize_tuple = s.serialize_tuple(COMPRESSED_SIZE)?;
    let bytes = g2.to_affine().to_compressed();
    for byte in &bytes {
        serialize_tuple.serialize_element(byte)?;
    }
    serialize_tuple.end()
}

struct TupleVisitor;

impl<'de> Visitor<'de> for TupleVisitor {
    type Value = G2Projective;
    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "a tuple of size {}", COMPRESSED_SIZE)
    }
    #[inline]
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let mut compressed: [u8; 96] = [0; 96];
        for (i, byte) in compressed.as_mut().iter_mut().enumerate() {
            let len_err = || DeserializeError::invalid_length(i, &self);
            *byte = seq.next_element()?.ok_or_else(len_err)?;
        }
        let g2_affine = G2Affine::from_compressed(&compressed);
        if g2_affine.is_none().into() {
            Err(DeserializeError::custom(DESERIALIZE_ERROR_MSG))
        } else {
            let g2_affine = g2_affine.unwrap();
            Ok(g2_affine.into())
        }
    }
}

pub fn deserialize<'de, D>(d: D) -> Result<G2Projective, D::Error>
where
    D: Deserializer<'de>,
{
    d.deserialize_tuple(COMPRESSED_SIZE, TupleVisitor)
}
