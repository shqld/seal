#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use serde::{Serialize, Deserialize};

use crate::check::check as check_impl;

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
    // Set panic hook for better error messages
    console_error_panic_hook::set_once();
    
    // Wrap in catch_unwind to handle panics gracefully
    let result = std::panic::catch_unwind(|| {
        check_impl(source)
    });
    
    let check_result = match result {
        Ok(result) => {
            let errors: Vec<WasmErrorInfo> = result.errors.into_iter().map(|error| {
                WasmErrorInfo {
                    message: error.message,
                    start_line: error.start_line,
                    start_column: error.start_column,
                    end_line: error.end_line,
                    end_column: error.end_column,
                }
            }).collect();
            
            WasmCheckResult {
                success: errors.is_empty(),
                errors,
            }
        },
        Err(panic_info) => {
            // Handle panic - convert to error
            let panic_msg = if let Some(s) = panic_info.downcast_ref::<&str>() {
                format!("Internal error (panic): {}", s)
            } else if let Some(s) = panic_info.downcast_ref::<String>() {
                format!("Internal error (panic): {}", s)
            } else {
                "Internal error: Unknown panic occurred".to_string()
            };
            
            WasmCheckResult {
                success: false,
                errors: vec![WasmErrorInfo {
                    message: panic_msg,
                    start_line: 1,
                    start_column: 1,
                    end_line: 1,
                    end_column: 1,
                }],
            }
        }
    };
    
    serde_wasm_bindgen::to_value(&check_result).map_err(|e| JsValue::from_str(&e.to_string()))
}