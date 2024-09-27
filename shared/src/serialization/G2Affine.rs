use bls12_381::G2Affine;
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use serde::de::{self, Visitor};
use serde_with::{SerializeAs, DeserializeAs};

pub fn serialize<S: Serializer>(p: &G2Affine, serializer: S) -> Result<S::Ok, S::Error> {
    let a: [u8; 96] = p.to_compressed();
    serializer.serialize_bytes(&a)
}

struct G2AffineVisitor;

impl<'de> Visitor<'de> for G2AffineVisitor {
    type Value = G2Affine;

    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.write_str("a byte slice with 96 bytes")
    }

    fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E> where E: de::Error {
        if value.len() != 96 {
            Err(E::custom(format!("slice length is out of range: {}", value.len())))
        } else {
            let arr: &[u8; 96] = value.try_into().unwrap();
            let p = G2Affine::from_compressed(arr);
            if p.is_none().into() {
                Err(E::custom("not a valid curve point"))
            } else {
                let p = p.unwrap();
                if p.is_on_curve().into() {
                    Ok(p)
                } else {
                    Err(E::custom("point is not on curve"))
                }
            }
        }
    }
}

pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<G2Affine, D::Error> {
    deserializer.deserialize_bytes(G2AffineVisitor)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SerializableG2Affine {
    #[serde(with = "crate::serialization::G2Affine")]
    v: bls12_381::G2Affine,
}

impl SerializeAs<G2Affine> for SerializableG2Affine {
    fn serialize_as<S>(value: &G2Affine, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {  
        SerializableG2Affine::serialize(&SerializableG2Affine { v: *value }, serializer)
    }
}

impl<'de> DeserializeAs<'de, G2Affine> for SerializableG2Affine {
    fn deserialize_as<D>(deserializer: D) -> Result<G2Affine, D::Error> where D: Deserializer<'de> {  
        deserialize(deserializer)
    }
}

#[cfg(test)]
mod tests {
    use group::Curve;
    use bls12_381::G2Affine;

    use crate::external::util::rand_scalar;
    use crate::serialization::SerializableG2Affine;

    #[test]
    fn test_serialization() {
        let a = (G2Affine::generator() * rand_scalar()).to_affine();
        let serialized: Vec<u8> = postcard::to_stdvec(&SerializableG2Affine { v: a }).unwrap();
        let b: SerializableG2Affine = postcard::from_bytes(&serialized).unwrap();

        assert_eq!(a, b.v)
    }
}
