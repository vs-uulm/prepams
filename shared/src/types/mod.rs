pub mod credential;
pub mod prerequisites;


use std::collections::HashMap;
use std::fmt;
use std::error::Error;
use postcard::to_stdvec;
use rand::RngCore;
use sha2::{Sha256, Digest};
use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};

use ed25519_zebra::{Signature, SigningKey, VerificationKey};
use bls12_381::{G1Affine, Scalar};
use serde_with::serde_as;

use crate::bindings::issuer::Issuer;
use crate::pbss::{BlindedSignRequest, BlindedSignature, PublicKey, self, UnblindedSignature, RerandomizedProofResponse};
use crate::external::util::{as_u32, rand_scalar};
use crate::serialization::SerializableG1Affine;
use crate::serialization::SerializableScalar;
use crate::serialization::{input, output, from_js, convert};
use crate::types::credential::*;
use crate::types::prerequisites::*;
use crate::proofs::generic::{Transcript, GenericProof};
use crate::proofs::participation::{ParticipationProof, ParticipationProofInput, ParticipationProofSecrets};
use crate::proofs::payout::{MAX_INPUTS, PayoutProofInput};

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "std", derive(Error))]
pub enum ProofError {
    /// This error occurs when a proof failed to verify.
    #[cfg_attr(feature = "std", error("Proof verification failed."))]
    VerificationError,

    /// This error occurs when the proof encoding is malformed.
    #[cfg_attr(feature = "std", error("Proof data could not be parsed."))]
    InvalidError,
}

impl fmt::Display for ProofError {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for ProofError {}


#[wasm_bindgen]
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug)]
pub struct Participation {
    #[serde(with = "crate::serialization::Scalar")]
    pub(crate) id: Scalar,
    pub(crate) proof: GenericProof::<ParticipationProofInput, ()>,
}

#[wasm_bindgen]
#[allow(non_snake_case)]
impl Participation {
  #[wasm_bindgen(getter)]
  pub fn id(&self) -> String {
    SerializableScalar::to_string(&self.id)
  }

  #[wasm_bindgen(getter)]
  pub fn reward(&self) -> u8 {
    as_u32(&self.proof.inputs.reward) as u8
  }

  pub fn verify(&self) -> Result<bool, JsError> {
    let mut verifier_transcript = Transcript::new(b"participation");
    if !self.proof.verify::<ParticipationProofSecrets, ParticipationProof>(&mut verifier_transcript).is_ok() {
      Err(JsError::new("prerequisites failed"))?;
    }

    Ok(true)
  }

  #[wasm_bindgen]
  pub fn serialize(&self) -> Result<Vec<u8>, JsError> {
    output(&self)
  }

  #[wasm_bindgen]
  pub fn deserialize(data: &[u8]) -> Result<Participation, JsError> {
    input(data)
  }
}

#[wasm_bindgen]
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct ConfirmedParticipation {
    pub(crate) id: String,
    pub(crate) value: u8,
    #[serde(with = "crate::serialization::G1Affine")]
    pub(crate) tag: G1Affine,
    #[serde(with = "crate::serialization::Scalar")]
    pub(crate) study: Scalar,
    pub(crate) request: BlindedSignRequest,
    pub(crate) signature: Signature
}

impl std::hash::Hash for ConfirmedParticipation {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.id.hash(state);
    self.value.hash(state);
    self.tag.to_compressed().hash(state);
    self.study.to_bytes().hash(state);
    self.request.hash(state);
    self.signature.to_bytes().hash(state);
  }
}

#[wasm_bindgen]
#[allow(non_snake_case)]
impl ConfirmedParticipation {
  #[wasm_bindgen(getter)]
  pub fn value(&self) -> u8 {
    self.value
  }

  #[wasm_bindgen(getter)]
  pub fn tag(&self) -> String {
    SerializableG1Affine::to_string(&self.tag)
  }

  #[wasm_bindgen(getter)]
  pub fn id(&self) -> String {
    self.id.clone()
  }

  #[wasm_bindgen(getter)]
  pub fn study(&self) -> String {
    SerializableScalar::to_string(&self.study)
  }

  #[wasm_bindgen(getter)]
  pub fn request(&self) -> Result<Vec<u8>, JsError> {
    convert(to_stdvec(&self.request))
  }

  #[wasm_bindgen(getter)]
  pub fn signature(&self) -> Result<Vec<u8>, JsError> {
    output(self.signature)
  }

  #[wasm_bindgen]
  pub fn serialize(&self) -> Result<Vec<u8>, JsError> {
    output(&self)
  }

  #[wasm_bindgen]
  pub fn deserialize(data: &[u8]) -> Result<ConfirmedParticipation, JsError> {
    input(data)
  }

  #[wasm_bindgen]
  pub fn from(id: &str, tag: &str, study: &str, request: &[u8], signature: &[u8], value: u8) -> Result<ConfirmedParticipation, JsError> {
    Ok(ConfirmedParticipation {
      id: id.to_string(),
      tag: SerializableG1Affine::from_string(tag)?,
      study: SerializableScalar::from_string(study)?,
      request: input(request)?,
      signature: input(signature)?,
      value: value,
    })
  }
}

#[serde_as]
#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttributeConstraint {
  Range(u32, u32, u32),
  Element(u32, Vec<u32>)
}

#[wasm_bindgen]
#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
  #[serde(with = "crate::serialization::Scalar")]
  pub(crate) id: Scalar,
  pub(crate) name: String,
  pub(crate) summary: String,
  pub(crate) description: String,
  pub(crate) duration: String,
  pub(crate) reward: u8,
  pub(crate) webBased: bool,
  pub(crate) studyUrl: Option<String>,
  pub(crate) qualifier: Vec<Qualifier>,
  pub(crate) disqualifier: Vec<Qualifier>,
  pub(crate) constraints: Vec<AttributeConstraint>,
}

#[allow(non_snake_case)]
impl Resource {
  pub fn random(rng: impl RngCore) -> Self {
    Resource {
      id: <bls12_381::Scalar as ff::Field>::random(rng),
      name: "".to_string(),
      summary: "".to_string(),
      description: "".to_string(),
      duration: "".to_string(),
      reward: 1,
      webBased: false,
      studyUrl: None,
      qualifier: vec![],
      disqualifier: vec![],
      constraints: vec![]
    }
  }

  pub fn addQualifier(&mut self, id: Scalar, tags: Vec<G1Affine>) {
    self.qualifier.push(Qualifier { id, tags });
  }

  pub fn addDisqualifier(&mut self, id: Scalar, tags: Vec<G1Affine>) {
    self.disqualifier.push(Qualifier { id, tags });
  }

  pub fn addConstraint(&mut self, constraint: AttributeConstraint) {
    self.constraints.push(constraint);
  }
}

#[wasm_bindgen]
#[allow(non_snake_case)]
impl Resource {
  #[wasm_bindgen(constructor)]
  pub fn new(
    id: JsValue,
    name: &str,
    summary: &str,
    description: &str,
    duration: &str,
    reward: u8,
    webBased: bool,
    studyUrl: &str,
    qualifier: JsValue,
    disqualifier: JsValue,
    constraints: JsValue 
  ) -> Result<Resource, JsValue> {
    let (id, qualifier, disqualifier) = if id.is_null() {
      let q: Vec<String> = from_js(qualifier)?;
      let d: Vec<String> = from_js(disqualifier)?;
      (
        rand_scalar(),
        q.iter().map(|q| Qualifier {
          id: SerializableScalar::from_string(q).unwrap(),
          tags: vec![]
        }).collect(),
        d.iter().map(|q| Qualifier {
          id: SerializableScalar::from_string(q).unwrap(),
          tags: vec![]
        }).collect()
    )} else {
      let q: Vec<(String, Vec<String>)> = from_js(qualifier)?;
      let d: Vec<(String, Vec<String>)> = from_js(disqualifier)?;

      (
          SerializableScalar::from_string(&id.as_string().unwrap()).unwrap(),
          q.iter().map(|(q, tags)| Qualifier {
            id: SerializableScalar::from_string(q).unwrap(),
            tags: tags.iter().map(|t| SerializableG1Affine::from_string(t).unwrap()).collect()
          }).collect(),
          d.iter().map(|(q, tags)| Qualifier {
            id: SerializableScalar::from_string(q).unwrap(),
            tags: tags.iter().map(|t| SerializableG1Affine::from_string(t).unwrap()).collect()
          }).collect()
      )
    };
    let constraints: Vec<(u32, String, Vec<u32>)> = from_js(constraints)?;

    Ok(Resource {
      id: id,
      name: name.to_string(),
      summary: summary.to_string(),
      description: description.to_string(),
      duration: duration.to_string(),
      reward: reward,
      webBased: webBased,
      studyUrl: webBased.then_some(studyUrl.to_string()),
      qualifier: qualifier,
      disqualifier: disqualifier,
      constraints: constraints.iter().map(|(i, t, p)| match t.as_str() {
        "number" => AttributeConstraint::Range(*i, p[0] as u32, p[1] as u32),
        "select" => AttributeConstraint::Element(*i, p.clone()),
        _ => panic!()
      }).collect()
    })
  }

  pub fn updateReferences(&mut self, issuer: &Issuer) {
    let mut qmap: HashMap<[u8; 32], Qualifier> = self.qualifier.iter().map(|q| (q.id.to_bytes(), Qualifier { id: q.id.clone(), tags: vec![] })).collect();
    let mut dmap: HashMap<[u8; 32], Qualifier> = self.disqualifier.iter().map(|q| (q.id.to_bytes(), Qualifier { id: q.id.clone(), tags: vec![] })).collect();
    for entry in issuer.ledger.entries.iter() {
      if let Some(transaction) = &entry.transaction {
        let id = transaction.participation.study.to_bytes();
        if let Some(v) = qmap.get_mut(&id) {
          v.tags.push(transaction.participation.tag.clone());
        }
        if let Some(v) = dmap.get_mut(&id) {
          v.tags.push(transaction.participation.tag.clone());
        }
      }
    }
    self.qualifier = qmap.into_values().collect();
    self.disqualifier = dmap.into_values().collect();
  }

  #[wasm_bindgen(getter)]
  pub fn id(&self) -> String {
    SerializableScalar::to_string(&self.id)
  }

  #[wasm_bindgen(getter)]
  pub fn name(&self) -> String {
    self.name.to_owned()
  }

  #[wasm_bindgen(getter)]
  pub fn summary(&self) -> String {
    self.summary.to_owned()
  }

  #[wasm_bindgen(getter)]
  pub fn description(&self) -> String {
    self.description.to_owned()
  }

  #[wasm_bindgen(getter)]
  pub fn duration(&self) -> String {
    self.duration.to_owned()
  }

  #[wasm_bindgen(getter)]
  pub fn reward(&self) -> u8 {
    self.reward
  }

  #[wasm_bindgen(getter)]
  pub fn webBased(&self) -> bool {
    self.webBased
  }

  #[wasm_bindgen(getter)]
  pub fn studyUrl(&self) -> String {
    self.studyUrl.as_ref().unwrap_or(&String::from("")).to_owned()
  }

  #[wasm_bindgen(getter)]
  pub fn qualifier(&self) -> Result<JsValue, JsError> {
    let a: Vec<String> = self.qualifier.iter().map(|q| SerializableScalar::to_string(&q.id)).collect();
    convert(serde_wasm_bindgen::to_value(&a))
  }

  #[wasm_bindgen(getter)]
  pub fn disqualifier(&self) -> Result<JsValue, JsError> {
    let a: Vec<String> = self.disqualifier.iter().map(|q| SerializableScalar::to_string(&q.id)).collect();
    convert(serde_wasm_bindgen::to_value(&a))
  }

  #[wasm_bindgen(getter)]
  pub fn constraints(&self) -> Result<JsValue, JsError> {
    let constraints: Vec<(u32, String, Vec<u32>)> = self.constraints.iter().map(|attr| match attr {
      AttributeConstraint::Range(index, lower, upper) => (
        *index,
        "number".to_string(),
        vec![*lower, *upper]
      ),
      AttributeConstraint::Element(index, values) => (
        *index,
        "select".to_string(),
        values.clone()
      )
    }).collect();

    convert(serde_wasm_bindgen::to_value(&constraints))
  }

  #[wasm_bindgen]
  pub fn serialize(&self) -> Result<Vec<u8>, JsError> {
    output(&self)
  }

  #[wasm_bindgen]
  pub fn deserialize(data: &[u8]) -> Result<Resource, JsError> {
    input(data)
  }
}

#[wasm_bindgen]
#[allow(non_snake_case)]
#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct SignedResource {
    pub(crate) owner: String,
    pub(crate) resource: Resource,
    pub(crate) signature: Signature,
}

#[wasm_bindgen]
#[allow(non_snake_case)]
impl SignedResource {
  #[wasm_bindgen]
  pub fn serialize(&self) -> Result<Vec<u8>, JsError> {
    output(&self)
  }

  #[wasm_bindgen]
  pub fn deserialize(data: &[u8]) -> Result<SignedResource, JsError> {
    input(data)
  }

  #[wasm_bindgen(getter)]
  pub fn owner(&self) -> String {
    self.owner.to_string()
  }

  #[wasm_bindgen(getter)]
  pub fn resource(&self) -> Resource {
    self.resource.clone()
  }

  #[wasm_bindgen(getter)]
  pub fn signature(&self) -> Vec<u8> {
    let bytes: [u8; 64] = self.signature.into();
    bytes.to_vec()
  }
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct Transaction {
  pub(crate) participation: ConfirmedParticipation,
  pub(crate) coin: BlindedSignature
}

#[wasm_bindgen]
#[allow(non_snake_case)]
impl Transaction {
  #[wasm_bindgen(getter)]
  pub fn coin(&self) -> Result<Vec<u8>, JsError> {
    output(&self.coin)
  }
}

#[serde_as]
#[wasm_bindgen]
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct Payout {
  #[serde_as(as = "Vec<crate::serialization::SerializableScalar>")]
  pub(crate) nullifier: Vec<Scalar>,
  pub(crate) recipient: [u8; 32],
  pub(crate) value: u8
}

#[wasm_bindgen]
#[allow(non_snake_case)]
impl Payout {
  #[wasm_bindgen(getter)]
  pub fn value(&self) -> u8 {
    self.value
  }

  #[wasm_bindgen]
  pub fn serialize(&self) -> Result<Vec<u8>, JsError> {
    output(&self)
  }

  #[wasm_bindgen]
  pub fn deserialize(data: &[u8]) -> Result<Payout, JsError> {
    input(data)
  }
}

impl From<&GenericProof<PayoutProofInput, Vec<RerandomizedProofResponse>>> for Payout {
    fn from(proof: &GenericProof<PayoutProofInput, Vec<RerandomizedProofResponse>>) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(proof.inputs.target.as_bytes());
        hasher.update(proof.inputs.recipient.as_bytes());

        Payout {
          nullifier: proof.inputs.nullifier.clone(),
          recipient: hasher.finalize().into(),
          value: proof.inputs.value
        }
    }
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub enum LedgerEntryType {
  Transaction,
  Payout
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct LedgerEntry {
  pub(crate) previous: Signature,
  pub(crate) transaction: Option<Transaction>,
  pub(crate) payout: Option<Payout>,
  pub(crate) signature: Signature,
}

#[wasm_bindgen]
#[allow(non_snake_case)]
impl LedgerEntry {
  #[wasm_bindgen(getter)]
  pub fn entryType(&self) -> LedgerEntryType {
    if self.transaction.is_some() {
      LedgerEntryType::Transaction
    } else {
      LedgerEntryType::Payout
    }
  }

  #[wasm_bindgen(getter)]
  pub fn transaction(&self) -> Transaction {
    self.transaction.clone().unwrap()
  }

  #[wasm_bindgen(getter)]
  pub fn payout(&self) -> Payout {
    self.payout.clone().unwrap()
  }

  #[wasm_bindgen(getter)]
  pub fn signature(&self) -> Vec<u8> {
    let bytes: [u8; 64] = self.signature.into();
    bytes.to_vec()
  }

  pub fn fromTransaction(previous: &[u8], participation: &ConfirmedParticipation, coin: &[u8], signature: &[u8]) -> Result<LedgerEntry, JsError> {
    let tx = Transaction {
      participation: participation.clone(),
      coin: input(coin)?,
    };

    let p: [u8; 64] = previous.try_into().unwrap();
    let s: [u8; 64] = signature.try_into().unwrap();

    Ok(LedgerEntry {
      previous: p.into(),
      signature: s.into(),
      transaction: Some(tx),
      payout: None
    })
  }

  pub fn fromPayout(previous: &[u8], payout: &[u8], signature: &[u8]) -> Result<LedgerEntry, JsError> {
    let payout: Payout = input(payout)?;

    let p: [u8; 64] = previous.try_into().unwrap();
    let s: [u8; 64] = signature.try_into().unwrap();

    Ok(LedgerEntry {
      previous: p.into(),
      signature: s.into(),
      transaction: None,
      payout: Some(payout)
    })
  }

  #[wasm_bindgen]
  pub fn serialize(&self) -> Result<Vec<u8>, JsError> {
    output(&self)
  }

  pub fn deserialize(data: &[u8]) -> Result<LedgerEntry, JsError> {
    input(data)
  }
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Ledger {
  pub(crate) head: Signature,
  pub(crate) entries: Vec<LedgerEntry>
}

impl Default for Ledger {
  fn default() -> Self {
    Ledger { head: [0; 64].into(), entries: vec![] }
  }
}

#[allow(non_snake_case)]
impl Ledger {
  pub fn appendTransaction(&mut self, signingKey: &SigningKey, tx: Transaction) -> Result<LedgerEntry, JsError> {
    let mut data = "transaction:".as_bytes().to_vec();
    data.extend_from_slice(&self.head.to_bytes());
    data.append(&mut convert(to_stdvec(&tx))?);

    let signature: Signature = signingKey.sign(&data);
    let previous: Signature = self.head.into();
    self.head = signature.clone();

    let entry = LedgerEntry {
      previous,
      transaction: Some(tx),
      payout: None,
      signature
    };
    self.entries.push(entry.clone());

    Ok(entry)
  }

  pub fn appendPayout(&mut self, signingKey: &SigningKey, payout: Payout) -> Result<LedgerEntry, JsError> {
    let mut data = "payout:".as_bytes().to_vec();
    data.extend_from_slice(&self.head.to_bytes());
    data.extend_from_slice(&payout.recipient);
    data.extend_from_slice(&payout.value.to_be_bytes());
    for coin in &payout.nullifier {
      data.extend_from_slice(&coin.to_bytes());
    }

    let signature: Signature = signingKey.sign(&data);
    let previous: Signature = self.head.into();
    self.head = signature.clone();
    let entry = LedgerEntry {
      previous,
      transaction: None,
      payout: Some(payout),
      signature
    };
    self.entries.push(entry.clone());

    Ok(entry)
  }

  pub fn verify(&mut self, vk: &VerificationKey, entry: &LedgerEntry) -> Result<(), JsError> {
    let head: [u8; 64] = self.head.into();
    let data = match entry.entryType() {
      LedgerEntryType::Transaction => {
        let tx = entry.transaction.clone().unwrap();
        let mut data = "transaction:".as_bytes().to_vec();
        data.extend_from_slice(&head);
        data.append(&mut convert(to_stdvec(&tx))?);
        data
      },
      LedgerEntryType::Payout => {
        let payout = entry.payout.clone().unwrap();
        let mut data = "payout:".as_bytes().to_vec();
        data.extend_from_slice(&head);
        data.extend_from_slice(&payout.recipient);
        data.extend_from_slice(&payout.value.to_be_bytes());
        for coin in payout.nullifier {
          data.extend_from_slice(&coin.to_bytes());
        }
        data
      }
    };

    vk.verify(&entry.signature, &data)?;
    self.head = entry.signature.into();
    self.entries.push(entry.clone());

    Ok(())
  }

  pub fn serialize(&self) -> Result<Vec<u8>, JsError> {
    output(&self)
  }

  pub fn deserialize(data: &[u8]) -> Result<Ledger, JsError> {
    input(data)
  }
}

#[wasm_bindgen]
#[serde_as]
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct NullRequest {
  #[serde_as(as = "Vec<crate::serialization::SerializableScalar>")]
  pub(crate) s: Vec<Scalar>,
  #[serde_as(as = "Vec<crate::serialization::SerializableScalar>")]
  pub(crate) d: Vec<Scalar>,
  pub(crate) vk: PublicKey,
  pub(crate) request: Vec<BlindedSignRequest>
}

#[allow(non_snake_case)]
impl NullRequest {
  pub fn new(vk: &PublicKey, credential: &Credential) -> NullRequest {
    let mut rng = rand::thread_rng();

    let mut S = vec![];
    let mut D = vec![];
    let mut R = vec![];

    let m = vec![Scalar::zero()];

    for _ in 0..MAX_INPUTS {
      let s = vec![<bls12_381::Scalar as ff::Field>::random(&mut rng), credential.identity];
      let d = <bls12_381::Scalar as ff::Field>::random(&mut rng);
      R.push(pbss::Blind(vk, &m, &s, &d, &mut rng));
      S.push(s[0]);
      S.push(s[1]);
      D.push(d);
    }

    NullRequest { s: S, d: D, request: R, vk: vk.clone() }
  }
}

#[wasm_bindgen]
#[allow(non_snake_case)]
impl NullRequest {
  pub fn request(&self) -> Result<Vec<u8>, JsError> {
    output(&self.request)
  }

  pub fn unblind(mut self, nullResponse: &[u8]) -> Result<Vec<u8>, JsError> {
    let mut responses: Vec<BlindedSignature> = input(nullResponse)?;

    if responses.len() != MAX_INPUTS {
      Err(JsError::new("response length is invalid"))?;
    }

    let mut nulls: Vec<UnblindedSignature> = vec![];
    let m = vec![Scalar::zero()];

    for _ in 0..MAX_INPUTS {
      let sig = responses.pop().unwrap();
      let s1 = self.s.pop().unwrap();
      let s0 = self.s.pop().unwrap();
      let d = self.d.pop().unwrap();

      nulls.push(convert(pbss::Unblind(&self.vk, &sig, &m, &vec![s0, s1], &d))?);
    }

    output(nulls)
  }
}
