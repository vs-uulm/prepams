use bls12_381::G2Affine;
use serde::{Serialize, Deserialize, Serializer, Deserializer};

pub fn serialize<S: Serializer>(p: &Option<G2Affine>, serializer: S) -> Result<S::Ok, S::Error> {
    match p {
        Some(p) => Some(base64::encode_config(p.to_compressed(), base64::URL_SAFE_NO_PAD)).serialize(serializer),
        _ => None::<String>.serialize(serializer)
    }
}
pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<G2Affine>, D::Error> {
    let o: Option<String> = Deserialize::deserialize(deserializer)?;

    match o {
        None => Ok(None),
        Some(s) => {
            match base64::decode_config(s, base64::URL_SAFE_NO_PAD) {
                Err(e) => Err(serde::de::Error::custom(e.to_string())),
                Ok(vec) => {
                    let arr: [u8; 96] = vec.try_into().ok().ok_or(serde::de::Error::custom(""))?;
                    let p = G2Affine::from_compressed(&arr);

                    if p.is_none().into() {
                        Err(serde::de::Error::custom(""))
                    } else {
                        Ok(Some(p.unwrap()))
                    }
                }
            }
        }
    }
}
