use rand::thread_rng;
use bls12_381::Scalar;
use wasm_bindgen::prelude::*;

use postcard::to_stdvec;
use serde::{Serialize, Deserialize};
use ed25519_zebra::{SigningKey, VerificationKey};
use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::SeedableRng;

use crate::serialization::{input, output, convert};
use crate::pbss::{self, BlindedSignRequest, BlindedSignature, RerandomizedProofResponse};
use crate::types::*;
use crate::types::credential::*;
use crate::credential;
use crate::proofs::generic::{Transcript, GenericProof};
use crate::proofs::payout::{MAX_INPUTS, PayoutProof, PayoutProofInput, PayoutProofSecrets};

#[wasm_bindgen]
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct Issuer {
    attributes: usize,
    signingKey: SigningKey,
    publicKey: IssuerPublicKey,
    secretKey: IssuerSecretKey,
    creditSigningKey: pbss::SecretKey,
    creditVerificationKey: pbss::PublicKey,
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub(crate) ledger: Ledger
}

#[wasm_bindgen]
#[allow(non_snake_case)]
pub struct PayoutResult {
    target: String,
    recipient: String,
    entry: LedgerEntry
}

#[wasm_bindgen]
#[allow(non_snake_case)]
impl PayoutResult {
  #[wasm_bindgen(getter)]
  pub fn target(&self) -> String {
    self.target.clone()
  }

  #[wasm_bindgen(getter)]
  pub fn recipient(&self) -> String {
    self.recipient.clone()
  }

  #[wasm_bindgen(getter)]
  pub fn entry(&self) -> LedgerEntry {
    self.entry.clone()
  }
}

#[wasm_bindgen]
#[allow(non_snake_case)]
impl Issuer {
    #[wasm_bindgen(constructor)]
    pub fn new(attributes: usize, seed: &[u8]) -> Issuer {
        let mut rng = if seed.len() != 32 {
            ChaCha20Rng::from_rng(rand::thread_rng()).unwrap()
        } else {
            // generation from seed for evaluation
            let seed: [u8;32] = seed.try_into().unwrap();
            ChaCha20Rng::from_seed(seed)
        };
        let (ipk, isk) = credential::init(&mut rng, attributes);
        let (csk, cvk) = pbss::Gen(&mut rng, 2, 1, "payment");
        let sk = SigningKey::new(&mut rng);
        
        Issuer {
            attributes,
            signingKey: sk,
            publicKey: ipk,
            secretKey: isk,
            creditSigningKey: csk,
            creditVerificationKey: cvk,
            ledger: Ledger::default()
        }
    }

    #[wasm_bindgen(getter)]
    pub fn attributes(&self) -> usize {
        self.attributes
    }

    #[wasm_bindgen(getter)]
    pub fn publicKey(&self) -> Result<Vec<u8>, JsError> {
        output(&self.publicKey)
    }

    #[wasm_bindgen(getter)]
    pub fn verificationKey(&self) -> Result<Vec<u8>, JsError> {
        output(&self.creditVerificationKey)
    }

    #[wasm_bindgen(getter)]
    pub fn ledgerVerificationKey(&self) -> Result<Vec<u8>, JsError> {
        let lvk: [u8; 32] = VerificationKey::from(&self.signingKey).into();
        output(lvk)
    }

    #[wasm_bindgen(getter)]
    pub fn ledger(&self) -> Result<Vec<u8>, JsError> {
        output(&self.ledger)
    }

    #[wasm_bindgen(getter)]
    pub fn head(&self) -> Result<Vec<u8>, JsError> {
        output(&self.ledger.head)
    }

    #[wasm_bindgen]
    pub fn load(&mut self, data: &[u8]) -> Result<(), JsError> {
        let ledger: Ledger = input(&data)?;
        self.ledger = ledger;
        Ok(())
    }

    #[wasm_bindgen]
    pub fn serialize(&self) -> Result<Vec<u8>, JsError> {
        output(&self)
    }

    #[wasm_bindgen]
    pub fn deserialize(data: &[u8]) -> Result<Issuer, JsError> {
        input(data)
    }

    pub fn issueCredential(&self, request: &[u8]) -> Result<Vec<u8>, JsError> {
        let request: IssueRequest = input(request)?;
        let response = convert(credential::issue(&self.publicKey, &self.secretKey, &request))?;
        output(response)
    }

    pub fn checkResourceSignature(&self, resource: &SignedResource, publicKey: &[u8]) -> Result<bool, JsError> {
        let blob = output(resource.resource.clone())?;
        let mut data = "resource:".as_bytes().to_vec();
        data.extend_from_slice(&blob);

        let vk: VerificationKey = input(publicKey)?;

        Ok(vk.verify(&resource.signature, &data).is_ok())
    }

    pub fn issueReward(&mut self, participation: &ConfirmedParticipation, pk: &[u8], reward: u8) -> Result<LedgerEntry, JsError> {
        let mut data = participation.id.as_bytes().to_vec();
        let mut req = to_stdvec(&participation.request)?;
        data.append(&mut req);

        let vk = convert(VerificationKey::try_from(pk))?;
        if vk.verify(&participation.signature, &data).is_err() {
            Err(JsError::new("reward signature invalid"))?;
        }
        if participation.value != reward {
            Err(JsError::new("reward amount does not match study"))?;
        }

        let coin = pbss::Sign(&self.creditVerificationKey, &self.creditSigningKey, &participation.request, thread_rng())?;
        let tx = Transaction {
            participation: participation.clone(),
            coin
        };
        self.ledger.appendTransaction(&self.signingKey, tx)
    }

    pub fn issueNulls(&mut self, request: &[u8]) -> Result<Vec<u8>, JsError> {
        let requests: Vec<BlindedSignRequest> = input(request)?;
        let mut coins: Vec<BlindedSignature> = vec![];

        for req in requests {
            if req.m.len() != 1 {
                Err(JsError::new("request contains invalid amount of attributes"))?;
            }
            if req.m[0] != Scalar::zero() {
                Err(JsError::new("value of request is not zero"))?;
            }

            coins.push(pbss::Sign(&self.creditVerificationKey, &self.creditSigningKey, &req, thread_rng())?);
        }

        output(coins)
    }

    pub fn appendEntry(mut self, entry: LedgerEntry) -> Result<Issuer, JsError> {
        let vk = VerificationKey::from(&self.signingKey);
        self.ledger.verify(&vk, &entry)?;
        Ok(self)
    }

    pub fn checkPayoutRequest(&mut self, request: &[u8]) -> Result<PayoutResult, JsError> {
        let proof: GenericProof::<PayoutProofInput, Vec<RerandomizedProofResponse>> = input(request)?;

        if proof.inputs.inputs.len() != MAX_INPUTS {
            Err(JsError::new("invalid padding"))?;
        }

        if proof.inputs.cvk != self.creditVerificationKey {
            Err(JsError::new("invalid public key"))?;
        }

        let mut verifier_transcript = Transcript::new(b"payout");
        convert(proof.verify::<PayoutProofSecrets, PayoutProof>(&mut verifier_transcript))?;

        let mut data = "payout:".as_bytes().to_vec();
        let inputs = convert(to_stdvec(&proof.inputs))?;
        data.extend_from_slice(&inputs);

        let entry = self.ledger.appendPayout(&self.signingKey, Payout::from(&proof))?;

        Ok(PayoutResult {
            entry: entry,
            target: proof.inputs.target.clone(),
            recipient: proof.inputs.recipient.clone()
        })
    }
}
