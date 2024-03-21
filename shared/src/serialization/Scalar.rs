use bls12_381::Scalar;
use base64::{encode_config, decode_config};
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use serde_with::{SerializeAs, DeserializeAs};
use simple_error::SimpleError;

pub fn serialize<S: Serializer>(p: &Scalar, serializer: S) -> Result<S::Ok, S::Error> {
    encode_config(p.to_bytes(), base64::URL_SAFE_NO_PAD).serialize(serializer)
}
pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Scalar, D::Error> {
    let s: String = Deserialize::deserialize(deserializer)?;
    match decode_config(s, base64::URL_SAFE_NO_PAD) {
        Err(e) => Err(serde::de::Error::custom(e.to_string())),
        Ok(vec) => {
            let bytes: [u8; 32] = vec.try_into().ok().ok_or(serde::de::Error::custom(""))?;
            let p = Scalar::from_bytes(&bytes);

            if p.is_none().into() {
                Err(serde::de::Error::custom(""))
            } else {
                Ok(p.unwrap())
            }
        }
    }
}

#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct SerializableScalar {
    #[serde(with = "crate::serialization::Scalar")]
    v: bls12_381::Scalar,
}

impl From<SerializableScalar> for bls12_381::Scalar {
    fn from(p: SerializableScalar) -> bls12_381::Scalar {
        p.v
    }
}

impl From<bls12_381::Scalar> for SerializableScalar {
    fn from(p: bls12_381::Scalar) -> SerializableScalar {
        SerializableScalar { v: p }
    }
}

impl From<&bls12_381::Scalar> for SerializableScalar {
    fn from(p: &bls12_381::Scalar) -> SerializableScalar {
        SerializableScalar { v: *p }
    }
}

impl<'a> From<&'a SerializableScalar> for &'a bls12_381::Scalar {
    fn from(p: &SerializableScalar) -> &bls12_381::Scalar {
        &p.v
    }
}

impl SerializeAs<Scalar> for SerializableScalar {
    fn serialize_as<S>(value: &Scalar, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {  
        SerializableScalar::serialize(&SerializableScalar { v: *value }, serializer)
    }
}

impl<'de> DeserializeAs<'de, Scalar> for SerializableScalar {
    fn deserialize_as<D>(deserializer: D) -> Result<Scalar, D::Error> where D: Deserializer<'de> {  
        deserialize(deserializer)
    }
}

impl std::hash::Hash for SerializableScalar {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.v.to_bytes().hash(state);
    }
}


impl SerializableScalar {
    pub fn from_string(s: &str) -> Result<Scalar, SimpleError> {
        match decode_config(s, base64::URL_SAFE_NO_PAD) {
            Err(e) => Err(SimpleError::new(e.to_string())),
            Ok(vec) => {
                let bytes: [u8; 32] = vec.try_into().ok().ok_or(SimpleError::new(""))?;
                let p = Scalar::from_bytes(&bytes);

                if p.is_none().into() {
                    Err(SimpleError::new(""))
                } else {
                    Ok(p.unwrap())
                }
            }
        }
    }

    pub fn to_string(s: &Scalar) -> String {
        encode_config(s.to_bytes(), base64::URL_SAFE_NO_PAD)
    }
}
