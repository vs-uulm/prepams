use bls12_381::{G1Affine, G2Affine, Gt, Scalar};
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use serde::{Serialize, Deserialize};
use group::Curve;
use serde_with::serde_as;
use sha2::{Digest, Sha256};

use crate::types::ProofError;

#[serde_as]
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct IssuerPublicKey {
    #[serde(with = "crate::serialization::Gt")]
    pub pk: Gt,
    #[serde_as(as = "Vec<crate::serialization::SerializableG1Affine>")]
    pub a: Vec<G1Affine>
}

#[derive(Serialize, Deserialize)]
pub struct IssuerSecretKey {
    #[serde(with = "crate::serialization::Scalar")]
    pub sk: Scalar
}

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct IssueRequest {
    pub id: String,
    #[serde(with = "crate::serialization::G1Affine")]
    pub alpha: G1Affine,
    #[serde(with = "crate::serialization::G1Affine")]
    pub gamma: G1Affine,
    #[serde(with = "crate::serialization::Scalar")]
    pub z1: Scalar,
    #[serde(with = "crate::serialization::Scalar")]
    pub z2: Scalar,
    #[serde_as(as = "Vec<crate::serialization::SerializableScalar>")]
    pub attributes: Vec<Scalar>
}

#[serde_as]
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Credential {
    #[serde(with = "crate::serialization::Scalar")]
    pub sk: Scalar,
    #[serde(with = "crate::serialization::Scalar")]
    pub d: Scalar,

    pub id: String,
    #[serde(with = "crate::serialization::Scalar")]
    pub identity: Scalar,

    #[serde_as(as = "Option<crate::serialization::SerializableG1Affine>")]
    pub sigma_1: Option<G1Affine>,
    #[serde_as(as = "Option<crate::serialization::SerializableG1Affine>")]
    pub sigma_2: Option<G1Affine>,
    #[serde_as(as = "Option<crate::serialization::SerializableG2Affine>")]
    pub sigma_3: Option<G2Affine>,

    #[serde_as(as = "Vec<crate::serialization::SerializableG1Affine>")]
    pub attributes: Vec<G1Affine>,
    #[serde_as(as = "Vec<crate::serialization::SerializableScalar>")]
    pub values: Vec<Scalar>
}

impl Credential {
    pub fn is_signed(&self) -> bool {
        self.sigma_1.is_some()
    }

    pub fn derive_tag(&self, id: &Scalar) -> Result<G1Affine, ProofError> {
        let tmp = (self.sk + id).invert();
        if bool::from(tmp.is_none()) {
            Err(ProofError::InvalidError)
        } else {
            Ok((G1Affine::generator() * tmp.unwrap()).to_affine())
        }
    }

    pub fn derive_reward_rng(&self, id: &Scalar) -> ChaCha20Rng {
        let mut hasher = Sha256::new();
        hasher.update(b"reward");
        hasher.update(&self.sk.to_bytes());
        hasher.update(&id.to_bytes());

        let seed = hasher.finalize().into();
        ChaCha20Rng::from_seed(seed)
    }
}

#[derive(Serialize, Deserialize)]
pub struct IssueResponse {
    #[serde(with = "crate::serialization::G1Affine")]
    pub sigma_1: G1Affine,
    #[serde(with = "crate::serialization::G1Affine")]
    pub sigma_2: G1Affine,
    #[serde(with = "crate::serialization::G2Affine")]
    pub sigma_3: G2Affine
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuthenticationRequest {
    #[serde(with = "crate::serialization::Scalar")]
    pub id: Scalar,
    #[serde(with = "crate::serialization::G1Affine")]
    pub token: G1Affine,
    #[serde(with = "crate::serialization::G2Affine")]
    pub s2: G2Affine,
    #[serde(with = "crate::serialization::Gt")]
    pub e1: Gt,
    #[serde(with = "crate::serialization::Gt")]
    pub e2: Gt,
    #[serde(with = "crate::serialization::Scalar")]
    pub z1: Scalar,
    #[serde(with = "crate::serialization::Scalar")]
    pub z2: Scalar,
    #[serde_as(as = "Vec<crate::serialization::SerializableScalar>")]
    pub za: Vec<Scalar>,
    #[serde(with = "crate::serialization::G1Affine")]
    pub z3: G1Affine,
    #[serde(with = "crate::serialization::G1Affine")]
    pub vc: G1Affine,
    #[serde(with = "crate::serialization::G1Affine")]
    pub bp: G1Affine,
    #[serde(with = "crate::serialization::Scalar")]
    pub zv: Scalar,
}
