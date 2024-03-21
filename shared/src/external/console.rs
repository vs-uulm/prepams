use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}

#[cfg(target_family = "wasm")]
macro_rules! console_log {
    // Note that this is using the `log` function imported above during `bare_bones`
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[cfg(not(target_family = "wasm"))]
macro_rules! console_log {
    // Note that this is using the `log` function imported above during `bare_bones`
    ($($t:tt)*) => (println!($($t)*))
}

pub(crate) use console_log;
