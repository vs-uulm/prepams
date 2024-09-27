use wasm_bindgen::prelude::*;

use crate::serialization::convert;
extern crate console_error_panic_hook;

pub mod issuer;
pub mod organizer;
pub mod participant;

#[wasm_bindgen]
pub fn init() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn b64decode(input: String) -> Result<Vec<u8>, JsError> {
    convert(base64::decode_config(&input, base64::URL_SAFE_NO_PAD))
}

#[wasm_bindgen]
pub fn b64encode(input: &[u8]) -> String {
    base64::encode_config(&input, base64::URL_SAFE_NO_PAD)
}
