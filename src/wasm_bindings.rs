use crate::PdfCompressor;
use wasm_bindgen::prelude::*;
use web_sys::console;

#[wasm_bindgen]
pub struct WasmPdfCompressor(PdfCompressor);

#[wasm_bindgen]
impl WasmPdfCompressor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console::log_1(&"PdfCompressor initialized".into());
        WasmPdfCompressor(PdfCompressor::new())
    }

    #[wasm_bindgen]
    pub fn compress(&mut self, input: &[u8]) -> Result<Vec<u8>, JsValue> {
        console::log_1(&"Starting compression".into());
        self.0
            .compress(input)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
}
