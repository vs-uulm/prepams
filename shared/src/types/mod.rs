use wasm_bindgen::prelude::*;

use group::Curve;
use ed25519_zebra::Signature;
use bls12_381::{G1Affine, G1Projective, G2Affine, Gt, Scalar};
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use postcard::{from_bytes, to_stdvec};

use crate::zksnark::GenericProof;
use crate::external::error::ProofError;
use crate::prerequisites::PrerequisitesProofInput;
use crate::serialization::{input, output, convert};

#[derive(Serialize, Deserialize)]
pub struct IssuerPublicKey {
    #[serde(with = "crate::serialization::Gt")]
    pub pk: Gt
}

#[derive(Serialize, Deserialize)]
pub struct IssuerSecretKey {
    #[serde(with = "crate::serialization::Scalar")]
    pub sk: Scalar
}

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
pub struct ResourceIdentifier {
    pub id: Scalar
}

impl ResourceIdentifier {
    pub fn new() -> ResourceIdentifier {
        ResourceIdentifier { id: <bls12_381::Scalar as ff::Field>::random(rand::thread_rng()) }
    }
}

impl std::hash::Hash for ResourceIdentifier {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.to_bytes().hash(state);
    }
}

impl Serialize for ResourceIdentifier {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        crate::serialization::Scalar::serialize(&self.id, serializer)
    }
}

impl<'de> Deserialize<'de> for ResourceIdentifier {
    fn deserialize<D>(deserializer: D) -> Result<ResourceIdentifier, D::Error> where D: Deserializer<'de> {
        let id: Scalar = crate::serialization::Scalar::deserialize(deserializer)?;
        Ok(ResourceIdentifier { id })
    }
}

impl std::ops::Deref for ResourceIdentifier {
    type Target = Scalar;

    fn deref(&self) -> &Scalar {
        &self.id
    }
}

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
    pub z2: Scalar
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Credential {
    #[serde(with = "crate::serialization::Scalar")]
    pub sk: Scalar,
    #[serde(with = "crate::serialization::Scalar")]
    pub d: Scalar,

    #[serde(with = "crate::serialization::G1AffineOption")]
    pub sigma_1: Option<G1Affine>,
    #[serde(with = "crate::serialization::G1AffineOption")]
    pub sigma_2: Option<G1Affine>,
    #[serde(with = "crate::serialization::G2AffineOption")]
    pub sigma_3: Option<G2Affine>
}

impl Credential {
    pub fn is_signed(&self) -> bool {
        self.sigma_1.is_some()
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

#[derive(Serialize, Deserialize)]
pub struct AuthenticationRequest {
    pub id: ResourceIdentifier,
    #[serde(with = "crate::serialization::G1Affine")]
    pub token: G1Affine,
    #[serde(with = "crate::serialization::G1Affine")]
    pub s1: G1Affine,
    #[serde(with = "crate::serialization::G2Affine")]
    pub s2: G2Affine,
    #[serde(with = "crate::serialization::Gt")]
    pub e1: Gt,
    #[serde(with = "crate::serialization::Gt")]
    pub e2: Gt,
    #[serde(with = "crate::serialization::Scalar")]
    pub z1: Scalar,
    #[serde(with = "crate::serialization::G1Affine")]
    pub z2: G1Affine
}

#[derive(Clone,Debug)]
pub struct Tag {
    pub tag: G1Affine
}

impl Tag {
    pub fn new(credential: &Credential, id: &ResourceIdentifier) -> Result<Tag, ProofError> {
        let tmp = (credential.sk + id.id).invert();
        if bool::from(tmp.is_none()) {
            Err(ProofError::InvalidError)
        } else {
            Ok(Tag::from(G1Affine::generator() * tmp.unwrap()))
        }
    }

    pub fn randomize(&self, r: &Scalar) -> Tag {
        Tag::from(self.tag * r)
    }
}

impl std::hash::Hash for Tag {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.tag.to_compressed().hash(state);
    }
}

impl std::ops::Deref for Tag {
    type Target = G1Affine;

    fn deref(&self) -> &G1Affine {
        &self.tag
    }
}

impl Serialize for Tag {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        crate::serialization::G1Affine::serialize(&self.tag, serializer)
    }
}

impl<'de> Deserialize<'de> for Tag {
    fn deserialize<D>(deserializer: D) -> Result<Tag, D::Error> where D: Deserializer<'de> {
        let tag: G1Affine = crate::serialization::G1Affine::deserialize(deserializer)?;
        Ok(Tag { tag })
    }
}

impl From<G1Projective> for Tag {
    fn from(tag: G1Projective) -> Self {
        Tag { tag: tag.to_affine() }
    }
}

impl From<G1Affine> for Tag {
    fn from(tag: G1Affine) -> Self {
        Tag { tag }
    }
}
impl PartialEq for Tag {
    fn eq(&self, other: &Self) -> bool {
        self.tag == other.tag
    }
}
impl Eq for Tag {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Qualifier {
    pub(crate) id: ResourceIdentifier,
    pub(crate) tags: Vec<Tag>
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RandomizedDisqualifier {
    pub(crate) id: ResourceIdentifier,
    pub(crate) randomized_tag: Tag,
    pub(crate) tags: Vec<Tag>
}

#[wasm_bindgen]
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct Reward {
    pub(crate) value: u8,
    pub(crate) tag: Tag,
    pub(crate) id: ResourceIdentifier,
    #[serde(with = "crate::serialization::G1Affine")]
    pub(crate) key: bls12_381::G1Affine,
    pub(crate) signature: Signature
}

#[wasm_bindgen]
#[allow(non_snake_case)]
impl Reward {
    pub fn serialize(&self) -> Result<JsValue, JsValue> {
        output(&self)
    }

    pub fn deserialize(o: JsValue) -> Result<Reward, JsValue> {
        input(o)
    }

    pub fn serializeBase64(&self) -> Result<JsValue, JsValue> {
        let vec = convert(to_stdvec(&self))?;
        Ok(base64::encode_config(&vec, base64::URL_SAFE_NO_PAD).into())
    }

    pub fn deserializeBase64(data: &str) -> Result<Reward, JsValue> {
        let vec: Vec<u8> = convert(base64::decode_config(data, base64::URL_SAFE_NO_PAD))?;
        convert(from_bytes(&vec))
    }

    pub fn serializeBinary(&self) -> Result<Vec<u8>, JsValue> {
        convert(to_stdvec(&self))
    }

    pub fn deserializeBinary(data: &[u8]) -> Result<Reward, JsValue> {
        convert(from_bytes(&data))
    }
}

#[wasm_bindgen]
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct Participation {
    pub(crate) id: ResourceIdentifier,

    pub(crate) authenticationProof: AuthenticationRequest,
    pub(crate) prerequisitesProof: GenericProof::<PrerequisitesProofInput>,

    #[serde(with = "crate::serialization::G1Affine")]
    pub(crate) rewardKey: bls12_381::G1Affine,
    pub(crate) reward: u8
}

#[wasm_bindgen]
#[allow(non_snake_case)]
impl Participation {
    pub fn serialize(&self) -> Result<JsValue, JsValue> {
        output(&self)
    }

    pub fn deserialize(o: JsValue) -> Result<Participation, JsValue> {
        input(o)
    }

    pub fn serializeBase64(&self) -> Result<JsValue, JsValue> {
        let vec = convert(to_stdvec(&self))?;
        Ok(base64::encode_config(&vec, base64::URL_SAFE_NO_PAD).into())
    }

    pub fn deserializeBase64(data: &str) -> Result<Participation, JsValue> {
        let vec: Vec<u8> = convert(base64::decode_config(data, base64::URL_SAFE_NO_PAD))?;
        convert(from_bytes(&vec))
    }

    pub fn serializeBinary(&self) -> Result<Vec<u8>, JsValue> {
        convert(to_stdvec(&self))
    }

    pub fn deserializeBinary(data: &[u8]) -> Result<Participation, JsValue> {
        convert(from_bytes(&data))
    }
}

#[wasm_bindgen]
#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Resource {
    pub(crate) id: ResourceIdentifier,
    pub(crate) qualifier: Vec<Qualifier>,
    pub(crate) disqualifier: Vec<Qualifier>,
    pub(crate) reward: u8
}

#[wasm_bindgen]
#[allow(non_snake_case)]
impl Resource {
    #[wasm_bindgen(constructor)]
    pub fn new(reward: u8) -> Result<Resource, JsValue> {
        Ok(Resource {
            id: ResourceIdentifier::new(),
            qualifier: vec![],
            disqualifier: vec![],
            reward
        })
    }

    pub fn serialize(&self) -> Result<JsValue, JsValue> {
        output(&self)
    }

    pub fn deserialize(o: JsValue) -> Result<Resource, JsValue> {
        input(o)
    }

    pub fn serializeBase64(&self) -> Result<JsValue, JsValue> {
        let vec = convert(to_stdvec(&self))?;
        Ok(base64::encode_config(&vec, base64::URL_SAFE_NO_PAD).into())
    }

    pub fn deserializeBase64(data: &str) -> Result<Resource, JsValue> {
        let vec: Vec<u8> = convert(base64::decode_config(data, base64::URL_SAFE_NO_PAD))?;
        convert(from_bytes(&vec))
    }

    pub fn serializeBinary(&self) -> Result<Vec<u8>, JsValue> {
        convert(to_stdvec(&self))
    }

    pub fn deserializeBinary(data: &[u8]) -> Result<Resource, JsValue> {
        convert(from_bytes(&data))
    }

    #[wasm_bindgen(getter)]
    pub fn id(&self) -> Result<JsValue, JsValue> {
        output(self.id)
    }
}

#[derive(Default, Debug, PartialEq, Eq, Copy, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub(crate) value: u8,
    #[serde(with = "crate::serialization::G1Affine")]
    pub(crate) pk: G1Affine,
    pub(crate) id: ResourceIdentifier
}

impl std::hash::Hash for Transaction {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state);
        self.pk.to_compressed().hash(state);
        self.id.id.to_bytes().hash(state);
    }
}
