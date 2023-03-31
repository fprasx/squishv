use wasm_bindgen::prelude::*;

pub mod executor;
pub mod parse;
pub mod parse_v2;

use crate::parse::Instruction;

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
    serde_wasm_bindgen::to_value(&parsed)
        .unwrap_or(serde_wasm_bindgen::to_value(&Vec::<usize>::new()).unwrap())
}

#[wasm_bindgen]
pub fn parse_instruction(program: String) -> JsValue {
    serde_wasm_bindgen::to_value(&program.parse::<Instruction>())
        .unwrap_or(serde_wasm_bindgen::to_value(&Vec::<usize>::new()).unwrap())
}
