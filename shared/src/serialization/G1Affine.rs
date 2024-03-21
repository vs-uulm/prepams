use bls12_381::G1Affine;
use base64::{encode_config, decode_config};
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use serde_with::{SerializeAs, DeserializeAs};
use simple_error::SimpleError;

pub fn serialize<S: Serializer>(p: &G1Affine, serializer: S) -> Result<S::Ok, S::Error> {
    to_string(p).serialize(serializer)
}
pub fn to_string(p: &G1Affine) -> String {
    base64::encode_config(p.to_compressed(), base64::URL_SAFE_NO_PAD)
}
pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<G1Affine, D::Error> {
    let s: String = Deserialize::deserialize(deserializer)?;
    match base64::decode_config(s, base64::URL_SAFE_NO_PAD) {
        Err(e) => Err(serde::de::Error::custom(e.to_string())),
        Ok(vec) => {
            let arr: [u8; 48] = vec.try_into().ok().ok_or(serde::de::Error::custom(""))?;
            let p = G1Affine::from_compressed(&arr);

            if p.is_none().into() {
                Err(serde::de::Error::custom(""))
            } else {
                Ok(p.unwrap())
            }
        }
    }
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
