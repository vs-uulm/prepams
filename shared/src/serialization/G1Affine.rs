use bls12_381::G1Affine;
use serde::{Serialize, Deserialize, Serializer, Deserializer};

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
