
use wasm_bindgen::prelude::*;

use serde::{Serialize, Deserialize};
use postcard::{from_bytes, to_stdvec};
use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::SeedableRng;
use ed25519_zebra::{SigningKey, VerificationKey};

use crate::types::*;
use crate::serialization::{input, output, convert};
use crate::prerequisites::{PrerequisitesProof, PrerequisitesProofSecrets};
use crate::zksnark::{Transcript};

#[wasm_bindgen]
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct Organizer {
    identity: String,
    secretKey: SigningKey,
    issuerPublicKey: IssuerPublicKey
}

#[wasm_bindgen]
#[allow(non_snake_case)]
impl Organizer {
    pub fn serialize(&self) -> Result<JsValue, JsValue> {
        output(&self)
    }

    pub fn deserialize(o: JsValue) -> Result<Organizer, JsValue> {
        input(o)
    }

    pub fn serializeBase64(&self) -> Result<JsValue, JsValue> {
        let vec = convert(to_stdvec(&self))?;
        Ok(base64::encode_config(&vec, base64::URL_SAFE_NO_PAD).into())
    }

    pub fn deserializeBase64(data: &str) -> Result<Organizer, JsValue> {
        let vec: Vec<u8> = convert(base64::decode_config(data, base64::URL_SAFE_NO_PAD))?;
        convert(from_bytes(&vec))
    }

    pub fn serializeBinary(&self) -> Result<Vec<u8>, JsValue> {
        convert(to_stdvec(&self))
    }

    pub fn deserializeBinary(data: &[u8]) -> Result<Organizer, JsValue> {
        convert(from_bytes(&data))
    }

    #[wasm_bindgen(constructor)]
    pub fn new(identity: &str, issuerPublicKey: JsValue, seed: &[u8]) -> Result<Organizer, JsValue> {
        let ipk: IssuerPublicKey = input(issuerPublicKey)?;

        let seed: [u8; 32] = convert(seed.try_into())?;
        let mut rng = ChaCha20Rng::from_seed(seed);
        let secretKey = SigningKey::new(&mut rng);

        Ok(Organizer {
            identity: identity.to_string(),
            secretKey,
            issuerPublicKey: ipk
        })
    }

    #[wasm_bindgen(getter)]
    pub fn publicKey(&self) -> JsValue {
        base64::encode_config(&VerificationKey::from(&self.secretKey), base64::URL_SAFE_NO_PAD).into()
    }

    #[wasm_bindgen(getter)]
    pub fn role(&self) -> JsValue {
        JsValue::from("organizer")
    }

    #[wasm_bindgen(getter)]
    pub fn identity(&self) -> Result<JsValue, JsValue> {
        output(&self.identity)
    }

    #[wasm_bindgen(getter)]
    pub fn id(&self) -> Result<JsValue, JsValue> {
        output(&self.identity)
    }

    pub fn checkParticipation(&self, participation: &Participation) -> Result<bool, JsValue> {
        if !crate::credential::verify(&self.issuerPublicKey, &participation.authenticationProof) {
            Err("authentication failed")?;
        }

        if participation.authenticationProof.token != participation.prerequisitesProof.inputs.tag.tag {
            Err("authentication and prerequisites tag mismatch")?;
        }

        let mut verifier_transcript = Transcript::new(b"prerequisites");
        if !participation.prerequisitesProof.verify::<PrerequisitesProofSecrets, PrerequisitesProof>(&mut verifier_transcript).is_ok() {
            Err("prerequisites failed")?;
        }

        // check if tag matches
        Ok(true)
    }

    pub fn signResource(&self, resource: &str) -> Result<String, JsValue> {
        let mut data = "resource:".as_bytes().to_vec();
        data.extend_from_slice(resource.as_bytes());

        let sig: [u8; 64] = self.secretKey.sign(&data).into();
        Ok(base64::encode_config(&sig, base64::URL_SAFE_NO_PAD))
    }

    pub fn issueReward(&self, participation: Participation) -> Result<Reward, JsValue> {
        let mut reward = participation.rewardKey.to_compressed().to_vec();
        reward.extend_from_slice(&participation.id.to_bytes());
        reward.push(participation.reward);

        let sig = self.secretKey.sign(&reward);

        Ok (Reward {
            id: participation.id,
            tag: Tag::from(participation.authenticationProof.token),
            value: participation.reward,
            key: participation.rewardKey,
            signature: sig
        })
    }
}
