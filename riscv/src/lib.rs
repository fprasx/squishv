use wasm_bindgen::prelude::*;

pub mod executor;
pub mod lex;
pub mod parse;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn helloworld() -> JsValue {
    serde_wasm_bindgen::to_value(&"Hello from Rust!").unwrap()
}

// vec! like syntax for a hashmap
#[macro_export]
macro_rules! map {
    ($($key:expr => $val:expr),* $(,)?) => {
        ::std::collections::HashMap::from([$(($key, $val),)*])
    };
}
