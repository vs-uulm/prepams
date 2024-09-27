use bls12_381::{Gt, Scalar};
use serde::de::{self, Visitor};
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use serde_with::{SerializeAs, DeserializeAs};

const LEN: usize = std::mem::size_of::<Gt>();

pub fn serialize<S: Serializer>(p: &Gt, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_bytes(&gt_to_bytes(&p))
}

struct GtVisitor;

impl<'de> Visitor<'de> for GtVisitor {
    type Value = Gt;

    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.write_fmt(format_args!("a byte slice with {} bytes", LEN))
    }

    fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E> where E: de::Error {
        if value.len() != LEN {
            Err(E::custom(format!("slice length is out of range: {}", value.len())))
        } else {
            let arr: [u8; LEN] = value.try_into().unwrap();

            unsafe {
                let target: Gt = std::mem::transmute(arr);
                if (target * (-Scalar::one()) + target) == Gt::identity() {
                    Ok(target)
                } else {
                    Err(E::custom("point is not on curve"))
                }
            }
        }
    }
}

pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Gt, D::Error> {
    deserializer.deserialize_bytes(GtVisitor)
}

pub fn gt_to_bytes(a: &Gt) -> [u8; LEN] {
    unsafe {
        std::mem::transmute(*a)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SerializableGt {
    #[serde(with = "crate::serialization::Gt")]
    v: Gt,
}

impl From<Gt> for SerializableGt {
    fn from(p: Gt) -> SerializableGt {
        SerializableGt { v: p }
    }
}

impl SerializeAs<Gt> for SerializableGt {
    fn serialize_as<S>(value: &Gt, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        SerializableGt::serialize(&SerializableGt { v: *value }, serializer)
    }
}

impl<'de> DeserializeAs<'de, Gt> for SerializableGt {
    fn deserialize_as<D>(deserializer: D) -> Result<Gt, D::Error> where D: Deserializer<'de> {
        deserialize(deserializer)
    }
}

#[cfg(test)]
mod tests {
    use group::Curve;
    use bls12_381::{pairing, G1Affine, G2Affine};

    use crate::external::util::rand_scalar;
    use crate::serialization::SerializableGt;

    #[test]
    fn test_serialization() {
        let a = pairing(&(G1Affine::generator() * rand_scalar()).to_affine(), &G2Affine::generator());
        let serialized: Vec<u8> = postcard::to_stdvec(&SerializableGt::from(a)).unwrap();
        let b: SerializableGt = postcard::from_bytes(&serialized).unwrap();

        assert_eq!(a, b.v)
    }
}
