#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use serde::{Serialize, Deserialize};

use crate::check::{check as check_impl, CheckError};

#[cfg(target_arch = "wasm32")]
#[derive(Serialize, Deserialize)]
pub struct WasmCheckResult {
    pub errors: Vec<WasmErrorInfo>,
    pub success: bool,
}

#[cfg(target_arch = "wasm32")]
#[derive(Serialize, Deserialize)]
pub struct WasmErrorInfo {
    pub message: String,
    pub start_line: u32,
    pub start_column: u32,
    pub end_line: u32,
    pub end_column: u32,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn type_check(source: &str) -> Result<JsValue, JsValue> {
    let result = check_impl(source);
    
    let errors: Vec<WasmErrorInfo> = result.errors.into_iter().map(|error| {
        WasmErrorInfo {
            message: error.message,
            start_line: error.start_line,
            start_column: error.start_column,
            end_line: error.end_line,
            end_column: error.end_column,
        }
    }).collect();
    
    let check_result = WasmCheckResult {
        success: errors.is_empty(),
        errors,
    };
    
    serde_wasm_bindgen::to_value(&check_result).map_err(|e| JsValue::from_str(&e.to_string()))
}