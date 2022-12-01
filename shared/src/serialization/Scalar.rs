use bls12_381::Scalar;
use base64::{encode_config, decode_config};
use serde::{Serialize, Deserialize, Serializer, Deserializer};

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
