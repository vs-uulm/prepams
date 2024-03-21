use wasm_bindgen::prelude::*;

use postcard::to_stdvec;
use serde::{Serialize, Deserialize};
use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::SeedableRng;
use ed25519_zebra::{SigningKey, VerificationKey};

use crate::types::*;
use crate::types::credential::*;
use crate::serialization::{input, output, convert};

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
    #[wasm_bindgen(constructor)]
    pub fn new(identity: &str, issuerPublicKey: &[u8], seed: &[u8]) -> Result<Organizer, JsError> {
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
    pub fn issuerPublicKey(&self) -> Result<Vec<u8>, JsError> {
        output(&self.issuerPublicKey)
    }

    #[wasm_bindgen(setter)]
    pub fn set_issuerPublicKey(&mut self, issuerPublicKey: &[u8]) -> Result<(), JsError> {
        let ipk: IssuerPublicKey = input(issuerPublicKey)?;
        self.issuerPublicKey = ipk;
        Ok(())
    }

    #[wasm_bindgen(getter)]
    pub fn publicKey(&self) -> Vec<u8> {
        let vk_bytes: [u8; 32] = VerificationKey::from(&self.secretKey).into();
        vk_bytes.to_vec()
    }

    #[wasm_bindgen(getter)]
    pub fn role(&self) -> String {
        "organizer".to_string()
    }

    #[wasm_bindgen(getter)]
    pub fn identity(&self) -> String {
        self.identity.to_string()
    }

    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String {
        self.identity.to_string()
    }

    pub fn serialize(&self) -> Result<Vec<u8>, JsError> {
        output(self)
    }

    pub fn deserialize(data: &[u8]) -> Result<Organizer, JsError> {
        input(data)
    }

    pub fn signResource(&self, resource: &Resource) -> Result<Vec<u8>, JsError> {
        let blob = output(resource)?;
        let mut data = "resource:".as_bytes().to_vec();
        data.extend_from_slice(&blob);

        let sig = self.secretKey.sign(&data);

        let signed = SignedResource {
            owner: self.identity.clone(),
            resource: resource.clone(), 
            signature: sig
        };

        let serialized: Vec<u8> = output(signed)?;
        let result: SignedResource = input(&serialized)?;
        output(&result)
    }

    pub fn confirmParticipation(&self, participation: &Participation, id: String) -> Result<Vec<u8>, JsError> {
        let mut data = id.as_bytes().to_vec();
        let mut req = to_stdvec(&participation.proof.inputs.reward_request)?;
        data.append(&mut req);

        let sig = self.secretKey.sign(&data);
        assert_eq!(self.issuerPublicKey, participation.proof.inputs.ipk);

        output(ConfirmedParticipation {
            id: id,
            study: participation.id,
            tag: participation.proof.inputs.tag,
            value: participation.reward(),
            request: participation.proof.inputs.reward_request.clone(),
            signature: sig
        })
    }
}
