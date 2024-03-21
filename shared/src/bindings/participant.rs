use std::collections::HashMap;

use bls12_381::Scalar;
use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};

use ed25519_zebra::VerificationKey;
use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::SeedableRng;

use crate::external::knapsack::knapsack;
use crate::external::util::as_scalar;
use crate::pbss;
use crate::pbss::{RerandomizedProofResponse, UnblindedSignature};
use crate::serialization::{input, output, convert, SerializableScalar};
use crate::types::*;
use crate::types::credential::*;

use crate::credential;
use crate::proofs::participation::{ParticipationProof, ParticipationProofInput, ParticipationProofSecrets};
use crate::proofs::payout::{PayoutProof, PayoutProofInput, PayoutProofSecrets};
use crate::proofs::generic::{Transcript, GenericProof};

#[wasm_bindgen]
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Clone)]
pub struct Participant {
    identity: String,
    attributes: Vec<u32>,
    credential: Option<Credential>,
    issuerPublicKey: Option<IssuerPublicKey>,
    creditVerificationKey: Option<pbss::PublicKey>,
    ledgerVerificationKey: VerificationKey
}

#[wasm_bindgen]
#[allow(non_snake_case)]
pub struct PayoutRequest {
    costs: u32,
    proof: GenericProof::<PayoutProofInput, Vec<RerandomizedProofResponse>>
}

#[wasm_bindgen]
#[allow(non_snake_case)]
impl PayoutRequest {
  #[wasm_bindgen(getter)]
  pub fn costs(&self) -> u32 {
    self.costs
  }

  #[wasm_bindgen(getter)]
  pub fn proof(&self) -> Result<Vec<u8>, JsError> {
    output(&self.proof)
  }
}

impl Participant {
    pub fn credential(&self) -> Option<&Credential> {
        self.credential.as_ref()
    }
}

#[wasm_bindgen]
#[allow(non_snake_case)]
impl Participant {
    #[wasm_bindgen(constructor)]
    pub fn new(identity: &str, attributes: &[u32], lvk: &[u8]) -> Participant {
        Participant {
            identity: identity.to_string(),
            attributes: attributes.to_vec(),
            credential: None,
            issuerPublicKey: None,
            creditVerificationKey: None,
            ledgerVerificationKey: VerificationKey::try_from(lvk).unwrap()
        }
    }

    #[wasm_bindgen(getter)]
    pub fn role(&self) -> String {
        "participant".to_string()
    }

    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String {
        self.identity.to_string()
    }

    #[wasm_bindgen(getter)]
    pub fn identity(&self) -> String {
        self.identity.to_string()
    }

    #[wasm_bindgen(getter)]
    pub fn attributes(&self) -> Vec<u32> {
        self.attributes.clone()
    }

    pub fn serialize(&self) -> Result<Vec<u8>, JsError> {
        output(self)
    }

    pub fn deserialize(data: &[u8]) -> Result<Participant, JsError> {
        input(data)
    }

    pub fn requestCredential(&mut self, issuerPublicKey: &[u8], creditVerificationKey: &[u8], seed: &[u8]) -> Result<Vec<u8>, JsError> {
        if self.credential.is_some() {
            Err(JsError::new("already requested credential"))?
        }

        if seed.len() != 32 {
            Err(JsError::new("invalid seed provided"))?
        }

        let seed: [u8;32] = seed.try_into().unwrap();
        let mut rng = ChaCha20Rng::from_seed(seed);

        let ipk: IssuerPublicKey = input(issuerPublicKey)?;
        let cvk: pbss::PublicKey = input(creditVerificationKey)?;

        let attributes = self.attributes.iter().map(|e| as_scalar(*e)).collect();

        let (request, credential) = credential::issue_request(&mut rng, &ipk, &self.identity, attributes);
        self.credential = Some(credential);
        self.issuerPublicKey = Some(ipk);
        self.creditVerificationKey = Some(cvk);

        // request
        output(request)
    }

    pub fn retrieveCredential(&mut self, issueResponse: &[u8]) -> Result<(), JsError> {
        let response: IssueResponse = input(issueResponse)?;
        match (&mut self.credential, &self.issuerPublicKey) {
            (Some(credential), Some(ipk)) => {
                if credential.is_signed() {
                    Err(JsError::new("credential already retrieved"))?;
                }

                credential::get_credential(&ipk, &response, credential)?;
                Ok(())
            },
            _ => Err(JsError::new("credential not yet requested"))?
        }
    }

    pub fn participate(&self, resource: &Resource) -> Result<Vec<u8>, JsError> {
        if let (Some(credential), Some(issuerPublicKey), Some(creditVerificationKey)) = (&self.credential, &self.issuerPublicKey, &self.creditVerificationKey) {
            if !credential.is_signed() {
                Err(JsError::new("credential not signed"))?;
            }

            let (inputs, secrets) = ParticipationProofInput::new(
                issuerPublicKey,
                creditVerificationKey,
                credential,
                resource,
            );

            let mut prover_transcript = Transcript::new(b"participation");
            let proof = GenericProof::<ParticipationProofInput, ()>::proove::<ParticipationProofSecrets, ParticipationProof>(&mut prover_transcript, inputs, secrets)?;

            let participation = Participation { id: resource.id, proof: proof };
            participation.serialize()
        } else {
            Err(JsError::new("credential not yet requested"))
        }
    }

    pub fn getBalance(&self, transactions: &[u8]) -> Result<JsValue, JsError> {
        let mut ledger: Ledger = Ledger::default();
        let transactions: Ledger = input(transactions)?;

        let mut participated: Vec<String> = vec![];
        let mut owned: HashMap<String, u8> = HashMap::new();

        if let Some(credential) = &self.credential {
            if !credential.is_signed() {
                Err(JsError::new("credential not signed"))?;
            }

            for entry in transactions.entries {
                // verify entry
                ledger.verify(&self.ledgerVerificationKey, &entry)?;

                match entry.entryType() {
                    LedgerEntryType::Payout => {
                        for coin in entry.payout.unwrap().nullifier {
                            owned.remove(&SerializableScalar::to_string(&coin));
                        }
                    },
                    LedgerEntryType::Transaction => {
                        let tx = entry.transaction.unwrap();
                        let tag = credential.derive_tag(&tx.participation.study)?;
                        if tx.participation.tag == tag {
                            participated.push(SerializableScalar::to_string(&tx.participation.study));
                            let mut rng = credential.derive_reward_rng(&tx.participation.study);
                            let s = <bls12_381::Scalar as ff::Field>::random(&mut rng);
                            owned.insert(SerializableScalar::to_string(&s), tx.participation.value);
                        }
                    }
                }
            }

            let balance = owned.iter().fold(0, |acc, (_, reward)| acc + *reward as u32);
            if cfg!(target_family = "wasm") {
                convert(serde_wasm_bindgen::to_value(&(balance, participated)))
            } else {
                Ok(JsValue::NULL)
            }
        } else {
            Err(JsError::new("credential not yet requested"))
        }
    }

    pub fn requestNulls(&self) -> Result<NullRequest, JsError> {
        if let (Some(credential), Some(creditVerificationKey)) = (&self.credential, &self.creditVerificationKey) {
            if !credential.is_signed() {
                Err(JsError::new("credential not signed"))?;
            }

            Ok(NullRequest::new(creditVerificationKey, &credential))
        } else {
            Err(JsError::new("credential not yet requested"))
        }
    }

    pub fn requestPayout(&self, amount: u8, target: &str, recipient: &str, nulls: &[u8], transactions: &[u8]) -> Result<PayoutRequest, JsError> {
        let nulls: Vec<UnblindedSignature> = input(nulls)?;

        let mut ledger: Ledger = Ledger::default();
        let transactions: Ledger = input(transactions)?;

        let mut owned: Vec<(Scalar, Scalar, Transaction)> = Vec::new();

        if let (Some(credential), Some(issuerPublicKey), Some(creditVerificationKey)) = (&self.credential, &self.issuerPublicKey, &self.creditVerificationKey) {
            if !credential.is_signed() {
                Err(JsError::new("credential not signed"))?;
            }

            for entry in transactions.entries {
                // verify entry
                ledger.verify(&self.ledgerVerificationKey, &entry)?;

                match entry.entryType() {
                    LedgerEntryType::Payout => {
                        for coin in entry.payout.unwrap().nullifier {
                            owned.retain(|(c, _, _)| coin != *c);
                        }
                    },
                    LedgerEntryType::Transaction => {
                        let tx = entry.transaction.unwrap();
                        let tag = credential.derive_tag(&tx.participation.study)?;
                        if tx.participation.tag == tag {
                            let mut rng = credential.derive_reward_rng(&tx.participation.study);
                            let s = <bls12_381::Scalar as ff::Field>::random(&mut rng);
                            let d = <bls12_381::Scalar as ff::Field>::random(&mut rng);
                            owned.push((s, d, tx));
                        }
                    }
                }
            }

            let unspend: Vec<usize> = owned.iter().map(|(_, _, tx)| tx.participation.value as usize).collect();
            let balance: usize = unspend.iter().sum();
            let remaining = balance - amount as usize;

            let mut costs = 0;
            let (_, items) = knapsack(remaining, unspend);
            let spend: Result<Vec<UnblindedSignature>, JsError> = owned.iter()
                .enumerate()
                .filter(|(i, _)| !items.contains(i))
                .map(|(_, (s, d, tx))| {
                    let m = vec![as_scalar(tx.participation.value as u32)];
                    let s = vec![s.clone(), credential.identity.clone()];
                    let d = d;
                    costs = costs + tx.participation.value;
                    convert(pbss::Unblind(creditVerificationKey, &tx.coin, &m, &s, &d))
                }).collect();
            let spend = spend?;

            let (inputs, secrets) = PayoutProofInput::new(
                issuerPublicKey,
                creditVerificationKey,
                amount,
                target,
                recipient,
                spend,
                nulls
            );

            let mut transcript = Transcript::new(b"payout");
            let proof = GenericProof::<PayoutProofInput, Vec<RerandomizedProofResponse>>::proove::<PayoutProofSecrets, PayoutProof>(&mut transcript, inputs, secrets)?;
            let costs = costs as u32;

            Ok(PayoutRequest { costs, proof })
        } else {
            Err(JsError::new("credential not yet requested"))
        }
    }
}
