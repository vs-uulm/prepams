use bls12_381::{Gt, Scalar};
use serde::{Serialize, Deserialize, Serializer, Deserializer};

const LEN: usize = std::mem::size_of::<Gt>();

pub fn serialize<S: Serializer>(p: &Gt, serializer: S) -> Result<S::Ok, S::Error> {
    base64::encode_config(p.to_bytes(), base64::URL_SAFE_NO_PAD).serialize(serializer)
}

pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Gt, D::Error> {
    let s: String = Deserialize::deserialize(deserializer)?;
    match base64::decode_config(s, base64::URL_SAFE_NO_PAD) {
        Err(e) => Err(serde::de::Error::custom(e.to_string())),
        Ok(vec) => {
            let buffer: Result<[u8; LEN], _> = vec.try_into();
            match buffer {
                Ok(d) => unsafe {
                    let target: Gt = std::mem::transmute(d);
                    if (target * (-Scalar::one()) + target) == Gt::identity() {
                        Ok(target)
                    } else {
                        Err(serde::de::Error::custom("not on curve"))
                    }
                },
                _ => Err(serde::de::Error::custom(""))
            }
        }
    }
}

pub trait SerializableGt {
    fn to_bytes(&self) -> [u8; LEN];
    fn to_base64(&self) -> String;
}

impl SerializableGt for Gt {
    fn to_bytes(&self) -> [u8; LEN] {
        unsafe {
            let target: [u8; LEN] = std::mem::transmute(*self);
            target
        }
    }

    fn to_base64(&self) -> String {
        base64::encode_config(self.to_bytes(), base64::URL_SAFE_NO_PAD)
    }
}
