use wasm_bindgen::{JsError, JsValue};

#[allow(non_snake_case)]
pub mod Scalar;
#[allow(non_snake_case)]
pub mod G1Affine;
#[allow(non_snake_case)]
pub mod G2Affine;
#[allow(non_snake_case)]
pub mod Gt;

pub use G1Affine::SerializableG1Affine;
pub use G2Affine::SerializableG2Affine;
pub use Scalar::SerializableScalar;
pub use Gt::SerializableGt;

pub fn convert<T, E>(o: Result<T, E>) -> Result<T, JsError> where E: std::fmt::Display {
    match o {
        Err(e) => Err (JsError::new(&e.to_string())),
        Ok(d) => Ok (d)
    }
}

pub fn output<T>(o: T) -> Result<Vec<u8>, JsError> where T: serde::Serialize {
    convert(postcard::to_stdvec(&o))
}

pub fn input<T>(i: &[u8]) -> Result<T, JsError> where T: for<'a> serde::de::Deserialize<'a> {
    convert(postcard::from_bytes(&i))
}

#[allow(dead_code)]
pub fn to_js<T>(o: T) -> Result<JsValue, JsError> where T: serde::Serialize {
    convert(serde_wasm_bindgen::to_value(&o))
}

pub fn from_js<T>(i: JsValue) -> Result<T, JsError> where T: for<'a> serde::de::Deserialize<'a> {
    convert(serde_wasm_bindgen::from_value(i))
}
