use std::collections::HashSet;

use wasm_bindgen::prelude::*;

use serde::{Serialize, Deserialize};
use postcard::{from_bytes, to_stdvec};
use ed25519_zebra::{SigningKey, VerificationKey, Signature};

use crate::types::*;
use crate::credential;
use crate::zksnark::{Transcript, GenericProof};
use crate::serialization::{input, output, convert};
use crate::payout::{MAX_INPUTS, PayoutProof, PayoutProofInput, PayoutProofSecrets};

#[wasm_bindgen]
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct Issuer {
    signingKey: SigningKey,
    publicKey: IssuerPublicKey,
    secretKey: IssuerSecretKey
}

#[wasm_bindgen]
#[allow(non_snake_case)]
impl Issuer {
    pub fn serialize(&self) -> Result<JsValue, JsValue> {
        output(&self)
    }

    pub fn deserialize(o: JsValue) -> Result<Issuer, JsValue> {
        input(o)
    }

    pub fn serializeBase64(&self) -> Result<JsValue, JsValue> {
        let vec = convert(to_stdvec(&self))?;
        Ok(base64::encode_config(&vec, base64::URL_SAFE_NO_PAD).into())
    }

    pub fn deserializeBase64(data: &str) -> Result<Issuer, JsValue> {
        let vec: Vec<u8> = convert(base64::decode_config(data, base64::URL_SAFE_NO_PAD))?;
        convert(from_bytes(&vec))
    }

    pub fn serializeBinary(&self) -> Result<Vec<u8>, JsValue> {
        convert(to_stdvec(&self))
    }

    pub fn deserializeBinary(data: &[u8]) -> Result<Issuer, JsValue> {
        convert(from_bytes(&data))
    }

    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<Issuer, JsValue> {
        let (ipk, isk) = credential::init(rand::thread_rng());
        let sk = SigningKey::new(rand::thread_rng());
        
        Ok(Issuer {
            signingKey: sk,
            publicKey: ipk,
            secretKey: isk
        })
    }

    #[wasm_bindgen(getter)]
    pub fn publicKey(&self) -> Result<JsValue, JsValue> {
        output(&self.publicKey)
    }

    pub fn issueCredential(&self, request: JsValue) -> Result<JsValue, JsValue> {
        let request: IssueRequest = input(request)?;
        let response = convert(credential::issue(&self.publicKey, &self.secretKey, &request))?;

        output(response)
    }

    pub fn checkRewardSignature(reward: Reward, approvedKeys: JsValue) -> Result<bool, JsValue> {
        let approvedKeys: Vec<String> = input(approvedKeys)?;

        let mut data = reward.key.to_compressed().to_vec();
        data.extend_from_slice(&reward.id.to_bytes());
        data.push(reward.value);

        let mut k : [u8; 32] = [0; 32];
        for key in approvedKeys {
            convert(base64::decode_config_slice(key, base64::URL_SAFE_NO_PAD, &mut k))?;
            let vk = convert(VerificationKey::try_from(k))?;
            if vk.verify(&reward.signature, &data).is_ok() {
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub fn checkResourceSignature(&self, resource: &str, signature: &str, publicKey: &str) -> Result<bool, JsValue> {
        let mut data = "resource:".as_bytes().to_vec();
        data.extend_from_slice(resource.as_bytes());

        let sig = convert(base64::decode_config(&signature, base64::URL_SAFE_NO_PAD))?;
        let pk = convert(base64::decode_config(&publicKey, base64::URL_SAFE_NO_PAD))?;

        let sig = TryInto::<[u8; 64]>::try_into(sig).ok().ok_or("invalid signature")?;
        let pk = TryInto::<[u8; 32]>::try_into(pk).ok().ok_or("invalid pk")?;

        let vk = convert(VerificationKey::try_from(pk))?;
        Ok(vk.verify(&Signature::from(sig), &data).is_ok())
    }

    pub fn checkPayoutRequest(&self, request: JsValue, transactions: JsValue, spend: JsValue) -> Result<JsValue, JsValue> {
        let proof: GenericProof::<PayoutProofInput> = input(request)?;
        let transactions: HashSet<Transaction> = input(transactions)?;
        let spend: HashSet<Tag> = input(spend)?;

        if !proof.inputs.transactions.iter().filter(|tx| tx.value == 0).count() != MAX_INPUTS {
            Err("invalid padding")?;
        }

        if !proof.inputs.transactions.iter().all(|tx| transactions.contains(tx) || tx.value == 0) {
            Err("transaction set contains unknown transactions")?;
        }

        if proof.inputs.nullifier.iter().any(|tx| spend.contains(tx)) {
            Err("used inputs are already spend")?;
        }

        let mut verifier_transcript = Transcript::new(b"payout");
        convert(proof.verify::<PayoutProofSecrets, PayoutProof>(&mut verifier_transcript))?;

        let mut data = "payout:".as_bytes().to_vec();
        let inputs = convert(to_stdvec(&proof.inputs))?;
        data.extend_from_slice(&inputs);

        let sig: [u8; 64] = self.signingKey.sign(&data).into();
        let receipt = base64::encode_config(&sig, base64::URL_SAFE_NO_PAD);

        output(receipt)
    }
}
