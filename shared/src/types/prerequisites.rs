use serde_with::serde_as;
use serde::{Serialize, Deserialize};
use bls12_381::{G1Affine, Scalar};

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Qualifier {
    #[serde(with = "crate::serialization::Scalar")]
    pub(crate) id: Scalar,
    #[serde_as(as = "Vec<crate::serialization::SerializableG1Affine>")]
    pub(crate) tags: Vec<G1Affine>
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RandomizedDisqualifier {
    #[serde(with = "crate::serialization::Scalar")]
    pub(crate) id: Scalar,
    #[serde(with = "crate::serialization::G1Affine")]
    pub(crate) randomized_tag: G1Affine,
    #[serde_as(as = "Vec<crate::serialization::SerializableG1Affine>")]
    pub(crate) tags: Vec<G1Affine>
}
