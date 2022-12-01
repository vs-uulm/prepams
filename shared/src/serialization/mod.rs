#[allow(non_snake_case)]
pub mod Scalar;
#[allow(non_snake_case)]
pub mod G1Affine;
#[allow(non_snake_case)]
pub mod G1AffineOption;
#[allow(non_snake_case)]
pub mod G2Affine;
#[allow(non_snake_case)]
pub mod G2AffineOption;
#[allow(non_snake_case)]
pub mod Gt;

pub fn convert<T, E>(o: Result<T, E>) -> Result<T, wasm_bindgen::JsValue> where E: std::fmt::Display {
    match o {
        Err(e) => Err (e.to_string().into()),
        Ok(d) => Ok (d)
    }
}

pub fn output<T>(o: T) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> where T: serde::Serialize {
    convert(wasm_bindgen::JsValue::from_serde(&o))
}

pub fn input<T>(i: wasm_bindgen::JsValue) -> Result<T, wasm_bindgen::JsValue> where T: for<'a> serde::de::Deserialize<'a> {
    convert(i.into_serde())
}