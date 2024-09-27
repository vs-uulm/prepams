use bls12_381::G1Affine;
use base64::{encode_config, decode_config};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{self, Visitor};
use serde_with::{SerializeAs, DeserializeAs};
use simple_error::SimpleError;

pub fn serialize<S: Serializer>(p: &G1Affine, serializer: S) -> Result<S::Ok, S::Error> {
    let a: [u8; 48] = p.to_compressed();
    serializer.serialize_bytes(&a)
}

struct G1AffineVisitor;

impl<'de> Visitor<'de> for G1AffineVisitor {
    type Value = G1Affine;

    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.write_str("a byte slice with 48 bytes")
    }

    fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E> where E: de::Error {
        if value.len() != 48 {
            Err(E::custom(format!("slice length is out of range: {}", value.len())))
        } else {
            let arr: &[u8; 48] = value.try_into().unwrap();
            let p = G1Affine::from_compressed(arr);
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

pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<G1Affine, D::Error> {
    deserializer.deserialize_bytes(G1AffineVisitor)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SerializableG1Affine {
    #[serde(with = "crate::serialization::G1Affine")]
    v: G1Affine,
}

impl From<G1Affine> for SerializableG1Affine {
    fn from(p: G1Affine) -> SerializableG1Affine {
        SerializableG1Affine { v: p }
    }
}

impl SerializeAs<G1Affine> for SerializableG1Affine {
    fn serialize_as<S>(value: &G1Affine, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {  
        SerializableG1Affine::serialize(&SerializableG1Affine { v: *value }, serializer)
    }
}

impl<'de> DeserializeAs<'de, G1Affine> for SerializableG1Affine {
    fn deserialize_as<D>(deserializer: D) -> Result<G1Affine, D::Error> where D: Deserializer<'de> {  
        deserialize(deserializer)
    }
}

impl SerializableG1Affine {
    pub fn from_string(s: &str) -> Result<G1Affine, SimpleError> {
        match decode_config(s, base64::URL_SAFE_NO_PAD) {
            Err(e) => Err(SimpleError::new(e.to_string())),
            Ok(vec) => {
                let bytes: [u8; 48] = vec.try_into().ok().ok_or(SimpleError::new(""))?;
                let p = G1Affine::from_compressed(&bytes);

                if p.is_none().into() {
                    Err(SimpleError::new(""))
                } else {
                    Ok(p.unwrap())
                }
            }
        }
    }

    #[allow(dead_code)]
    pub fn to_string(s: &G1Affine) -> String {
        encode_config(s.to_compressed(), base64::URL_SAFE_NO_PAD)
    }
}

#[cfg(test)]
mod tests {
    use group::Curve;
    use bls12_381::G1Affine;

    use crate::external::util::rand_scalar;
    use crate::serialization::SerializableG1Affine;

    #[test]
    fn test_serialization() {
        let a = (G1Affine::generator() * rand_scalar()).to_affine();
        let serialized: Vec<u8> = postcard::to_stdvec(&SerializableG1Affine::from(a)).unwrap();
        let b: SerializableG1Affine = postcard::from_bytes(&serialized).unwrap();

        assert_eq!(a, b.v)
    }
}
