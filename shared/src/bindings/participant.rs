use std::collections::HashMap;
use std::collections::HashSet;

use wasm_bindgen::prelude::*;

use group::Curve;
use serde::{Serialize, Deserialize};
use postcard::{from_bytes, to_stdvec};
use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::SeedableRng;

use crate::types::*;
use crate::credential;
use crate::serialization::{input, output, convert};
use crate::prerequisites::{PrerequisitesProof, PrerequisitesProofInput, PrerequisitesProofSecrets};
use crate::payout::{derive_reward_key, PayoutProof, PayoutProofInput, PayoutProofSecrets, PAYOUT_A};
use crate::zksnark::{Transcript, GenericProof};

#[wasm_bindgen]
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct Participant {
    identity: String,
    credential: Option<Credential>,
    issuerPublicKey: Option<IssuerPublicKey>
}

#[wasm_bindgen]
#[allow(non_snake_case)]
impl Participant {
    pub fn serialize(&self) -> Result<JsValue, JsValue> {
        output(&self)
    }

    pub fn deserialize(o: JsValue) -> Result<Participant, JsValue> {
        input(o)
    }

    pub fn serializeBase64(&self) -> Result<JsValue, JsValue> {
        let vec = convert(to_stdvec(&self))?;
        Ok(base64::encode_config(&vec, base64::URL_SAFE_NO_PAD).into())
    }

    pub fn deserializeBase64(data: &str) -> Result<Participant, JsValue> {
        let vec: Vec<u8> = convert(base64::decode_config(data, base64::URL_SAFE_NO_PAD))?;
        convert(from_bytes(&vec))
    }

    pub fn serializeBinary(&self) -> Result<Vec<u8>, JsValue> {
        convert(to_stdvec(&self))
    }

    pub fn deserializeBinary(data: &[u8]) -> Result<Participant, JsValue> {
        convert(from_bytes(&data))
    }

    #[wasm_bindgen(constructor)]
    pub fn new(identity: &str) -> Participant {
        Participant {
            identity: identity.to_string(),
            credential: None,
            issuerPublicKey: None
        }
    }

    pub fn data(&self) -> Result<JsValue, JsValue> {
        output(&self)
    }

    #[wasm_bindgen(getter)]
    pub fn role(&self) -> JsValue {
        JsValue::from("participant")
    }

    #[wasm_bindgen(getter)]
    pub fn id(&self) -> Result<JsValue, JsValue> {
        output(&self.identity)
    }

    #[wasm_bindgen(getter)]
    pub fn identity(&self) -> Result<JsValue, JsValue> {
        output(&self.identity)
    }

    pub fn requestCredential(&mut self, issuerPublicKey: JsValue, seed: &[u8]) -> Result<JsValue, JsValue> {
        if self.credential.is_some() {
            Err("already requested credential")?
        }

        let seed: [u8;32] = convert(seed.try_into())?;
        let mut rng = ChaCha20Rng::from_seed(seed);

        let ipk: IssuerPublicKey = input(issuerPublicKey)?;

        let (request, credential) = credential::issue_request(&mut rng, &ipk, &self.identity);
        self.credential = Some(credential);
        self.issuerPublicKey = Some(ipk);

        output(request)
    }

    pub fn retrieveCredential(&mut self, issueResponse: JsValue) -> Result<(), JsValue> {
        let response: IssueResponse = input(issueResponse)?;
        match (&mut self.credential, &self.issuerPublicKey) {
            (Some(credential), Some(ipk)) => {
                if credential.is_signed() {
                    Err("credential already retrieved")?;
                }

                credential::get_credential(&ipk, &response, credential)?;
                Ok(())
            },
            _ => Err("credential not yet requested")?
        }
    }

    pub fn participate(&self, resource: &Resource) -> Result<Participation, JsValue> {
        if let Some(credential) = &self.credential {
            if !credential.is_signed() {
                Err("credential not signed")?;
            }

            let (inputs, secrets) = PrerequisitesProofInput::new(
                credential,
                &resource.id,
                &resource.qualifier,
                &resource.disqualifier
            );

            let mut prover_transcript = Transcript::new(b"prerequisites");
            let proof = convert(GenericProof::<PrerequisitesProofInput>::proove::<PrerequisitesProofSecrets, PrerequisitesProof>(&mut prover_transcript, inputs, secrets))?;

            let mut verifier_transcript = Transcript::new(b"prerequisites");
            convert(proof.verify::<PrerequisitesProofSecrets, PrerequisitesProof>(&mut verifier_transcript))?;

            let request = credential::authenticate(&credential, &resource.id);

            let rewardKey = (bls12_381::G1Affine::generator() * derive_reward_key(&credential, &resource.id)).to_affine();

            let participation = Participation {
                id: resource.id,
                reward: resource.reward,
                authenticationProof: request,
                prerequisitesProof: proof,
                rewardKey: rewardKey
            };

            Ok(participation)
        } else {
            Err("credential not yet requested".into())
        }
    }

    pub fn getBalance(&self, transactions: JsValue, spend: JsValue) -> Result<JsValue, JsValue> {
        let transactions: Vec<Transaction> = input(transactions)?;
        let spend: HashSet<Tag> = input(spend)?;

        let A = PAYOUT_A();
        let G = bls12_381::G1Affine::generator();

        if let Some(credential) = &self.credential {
            if !credential.is_signed() {
                Err("credential not signed")?;
            }

            let mut balance: u32 = 0;
            let mut participated: HashMap<ResourceIdentifier, Tag> = HashMap::new();
            let mut keys: HashMap<ResourceIdentifier, bls12_381::Scalar> = HashMap::new();
            for tx in transactions {
                let sk = match keys.get(&tx.id) {
                    Some(sk) => sk,
                    _ => {
                        keys.insert(tx.id.clone(), derive_reward_key(&credential, &tx.id));
                        keys.get(&tx.id).unwrap()
                    }
                };

                if tx.pk == (G * sk).to_affine() {
                    let tag = Tag::from(A * sk.invert().unwrap());
                    if !spend.contains(&tag) {
                        balance += tx.value as u32;
                    }

                    participated.insert(tx.id.clone(), tag);
                }
            }

            output((balance, participated))
        } else {
            Err("credential not yet requested".into())
        }
    }

    pub fn requestPayout(&self, value: u8, target: &str, recipient: &str, transactions: JsValue, spend: JsValue) -> Result<JsValue, JsValue> {
        let transactions: Vec<Transaction> = input(transactions)?;
        let spend: HashSet<Tag> = input(spend)?;

        let A = PAYOUT_A();
        let g = bls12_381::G1Affine::generator();

        if let Some(credential) = &self.credential {
            if !credential.is_signed() {
                Err("credential not signed")?;
            }

            let mut keys: HashMap<ResourceIdentifier, bls12_381::Scalar> = HashMap::new();
            let unspend: Vec<&Transaction> = transactions.iter()
                .filter(|tx| {
                    let sk = match keys.get(&tx.id) {
                        Some(sk) => sk,
                        _ => {
                            keys.insert(tx.id.clone(), derive_reward_key(&credential, &tx.id));
                            keys.get(&tx.id).unwrap()
                        }
                    };
                    (tx.pk - g * sk).is_identity().into() && !spend.contains(&Tag::from(A * sk.invert().unwrap()))
                }).collect();
            drop(keys);

            let sum = unspend.iter().fold(0, |acc, tx| acc + tx.value as usize);

            // solve a 0/1 knapsack to find a set of spendable inputs that sum up to at least value
            let w = sum - value as usize;
            let n: usize = unspend.len();
            let mut m: Vec<Vec<(usize, Vec<&Transaction>)>> = vec![vec![(0, vec![]); w + 1]; n + 1];

            for i in 1..=n {
                for j in 1..=w {
                    m[i][j] = match j >= unspend[i - 1].value.into() && m[i - 1][j - (unspend[i - 1].value as usize)].0 + (unspend[i - 1].value as usize) > m[i - 1][j].0 {
                        true => (
                            (unspend[i - 1].value as usize) + m[i - 1][j - (unspend[i - 1].value as usize)].0,
                            m[i - 1][j - (unspend[i - 1].value as usize)].1.iter().chain(std::iter::once(&unspend[i - 1])).cloned().collect()
                        ),
                        false => m[i - 1][j].clone()
                    }
                }
            }

            let spend: Vec<Transaction> = unspend.iter().filter_map(|e| match m[n][w].1.contains(e) {
                false => Some(*e.clone()),
                true => None
            }).collect();
            drop(m);

            let cost = spend.iter().fold(0, |acc, tx| acc + tx.value as usize);

            let (inputs, secrets) = PayoutProofInput::new(
                credential,
                value,
                target,
                recipient,
                spend,
                transactions
            );

            let mut transcript = Transcript::new(b"payout");
            let proof = convert(GenericProof::<PayoutProofInput>::proove::<PayoutProofSecrets, PayoutProof>(&mut transcript, inputs, secrets))?;

            let mut verifier_transcript = Transcript::new(b"payout");
            convert(proof.verify::<PayoutProofSecrets, PayoutProof>(&mut verifier_transcript))?;

            output((cost, proof))
        } else {
            Err("credential not yet requested".into())
        }
    }
}
