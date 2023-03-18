use wasm_bindgen::prelude::*;

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


#[wasm_bindgen]
pub fn parse_riscv(program: String) -> JsValue {
    let parsed = parse::parse_program(program);
    serde_wasm_bindgen::to_value(&parsed).unwrap_or(serde_wasm_bindgen::to_value(&Vec::<usize>::new()).unwrap())
    // let x = serde_wasm_bindgen::to_value(&Vec::<usize>::new()).unwrap();
    // serde_wasm_bindgen::to_value(&program).unwrap()
    
}
