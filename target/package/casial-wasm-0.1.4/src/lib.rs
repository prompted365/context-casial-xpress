//! # Casial WASM
//!
//! WebAssembly bindings for universal consciousness-aware context coordination.
//! Enables deployment across browsers, edge computing, and any JavaScript environment.

use casial_core::{
    CasialEngine, CasialMission, CoordinationRequest,
    PerceptionId,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

// Set up memory allocator for WASM
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// Set up panic hook for better error messages
#[wasm_bindgen(start)]
pub fn main() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// JavaScript-friendly wrapper for the Casial Engine
#[wasm_bindgen]
pub struct CasialEngineWasm {
    engine: CasialEngine,
}

/// JavaScript-friendly coordination request
#[derive(Serialize, Deserialize)]
pub struct CoordinationRequestJs {
    pub tool_name: String,
    pub tool_args: serde_json::Value,
    pub environment: HashMap<String, String>,
    pub project_path: Option<String>,
    pub active_perceptions: Vec<String>, // Simplified as strings for JS
    pub paradox_tolerance: f64,
}

/// JavaScript-friendly coordination result
#[derive(Serialize, Deserialize)]
pub struct CoordinationResultJs {
    pub applied: bool,
    pub injected_content: String,
    pub modified_args: serde_json::Value,
    pub activated_rules: Vec<String>,
    pub used_templates: Vec<String>,
    pub paradoxes_detected: Vec<ParadoxReportJs>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// JavaScript-friendly paradox report
#[derive(Serialize, Deserialize)]
pub struct ParadoxReportJs {
    pub id: String,
    pub description: String,
    pub severity: String,
    pub resolution_strategy: String,
    pub confidence_impact: f64,
}

#[wasm_bindgen]
impl CasialEngineWasm {
    /// Create a new Casial engine for WASM
    #[wasm_bindgen(constructor)]
    pub fn new() -> CasialEngineWasm {
        CasialEngineWasm {
            engine: CasialEngine::new(),
        }
    }

    /// Load a mission from JSON string
    #[wasm_bindgen(js_name = loadMissionFromJson)]
    pub fn load_mission_from_json(&mut self, mission_json: &str) -> Result<(), JsValue> {
        let mission: CasialMission = serde_json::from_str(mission_json)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse mission JSON: {}", e)))?;

        self.engine
            .load_mission(mission)
            .map_err(|e| JsValue::from_str(&format!("Failed to load mission: {}", e)))?;

        Ok(())
    }

    /// Coordinate context for a tool request
    #[wasm_bindgen(js_name = coordinate)]
    pub fn coordinate(&mut self, request_json: &str) -> Result<String, JsValue> {
        let js_request: CoordinationRequestJs = serde_json::from_str(request_json)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse request: {}", e)))?;

        // Convert JS request to core request
        let core_request = CoordinationRequest {
            tool_name: js_request.tool_name,
            tool_args: js_request.tool_args,
            environment: js_request.environment.into_iter().collect(),
            project_path: js_request.project_path,
            active_perceptions: js_request
                .active_perceptions
                .iter()
                .map(|_| PerceptionId::new()) // Simplified conversion
                .collect(),
            paradox_tolerance: js_request.paradox_tolerance,
        };

        let result = self
            .engine
            .coordinate(core_request)
            .map_err(|e| JsValue::from_str(&format!("Coordination failed: {}", e)))?;

        // Convert result to JS-friendly format
        let js_result = CoordinationResultJs {
            applied: result.applied,
            injected_content: result.injected_content,
            modified_args: result.modified_args,
            activated_rules: result.activated_rules,
            used_templates: result.used_templates,
            paradoxes_detected: result
                .paradoxes_detected
                .iter()
                .map(|p| ParadoxReportJs {
                    id: p.id.to_string(),
                    description: p.description.clone(),
                    severity: "unknown".to_string(), // severity field not available
                    resolution_strategy: format!("{:?}", p.resolution_strategy),
                    confidence_impact: p.confidence_impact,
                })
                .collect(),
            metadata: result.metadata.into_iter().collect(),
        };

        serde_json::to_string(&js_result)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize result: {}", e)))
    }

    /// Get coordination history as JSON
    #[wasm_bindgen(js_name = getCoordinationHistory)]
    pub fn get_coordination_history(&self) -> String {
        let history = self.engine.get_coordination_history();
        serde_json::to_string(&history).unwrap_or_else(|_| "[]".to_string())
    }

    /// Get paradox registry as JSON
    #[wasm_bindgen(js_name = getParadoxRegistry)]
    pub fn get_paradox_registry(&self) -> String {
        let registry = self.engine.get_paradox_registry();
        serde_json::to_string(&registry).unwrap_or_else(|_| "[]".to_string())
    }

    /// Get engine statistics
    #[wasm_bindgen(js_name = getStatistics)]
    pub fn get_statistics(&self) -> String {
        let history = self.engine.get_coordination_history();
        let paradoxes = self.engine.get_paradox_registry();

        let stats = serde_json::json!({
            "coordination_events": history.len(),
            "total_paradoxes": paradoxes.len(),
            "consciousness_aware": true,
            "substrate_active": true,
            "paradox_resilient": true
        });

        serde_json::to_string(&stats).unwrap_or_else(|_| "{}".to_string())
    }
}

/// Utility functions for JavaScript integration
#[wasm_bindgen]
pub struct CasialUtils;

#[wasm_bindgen]
impl CasialUtils {
    /// Create a sample mission configuration for testing
    #[wasm_bindgen(js_name = createSampleMission)]
    pub fn create_sample_mission() -> String {
        let mission = serde_json::json!({
            "id": "sample-wasm-mission",
            "name": "Sample WASM Mission",
            "description": "A sample mission for WASM testing",
            "templates": [
                {
                    "id": "sample-template",
                    "name": "Sample Template",
                    "description": "A sample template for testing",
                    "categories": ["test"],
                    "priority": 10,
                    "enabled": true,
                    "content": "This is sample context content for testing WASM bindings.",
                    "perception_affinity": [],
                    "paradox_resistance": 0.8,
                    "metadata": {}
                }
            ],
            "rules": [
                {
                    "id": "sample-rule",
                    "name": "Sample Rule",
                    "enabled": true,
                    "conditions": {
                        "tool_patterns": ["test"],
                        "environment_vars": {},
                        "file_signals": [],
                        "perception_states": [],
                        "min_confidence": null
                    },
                    "actions": {
                        "template_ids": ["sample-template"],
                        "transform_type": "Prepend",
                        "target_field": null,
                        "char_limit": 1000,
                        "perception_lock": false
                    },
                    "perception_scope": [],
                    "paradox_handling": "Coexist"
                }
            ],
            "perceptions": [],
            "budgets": {
                "global_char_limit": 5000,
                "per_tool_limits": {},
                "perception_quotas": {},
                "paradox_overhead": 0.1
            },
            "created_at": "2025-01-01T00:00:00Z",
            "updated_at": "2025-01-01T00:00:00Z"
        });

        serde_json::to_string(&mission).unwrap_or_else(|_| "{}".to_string())
    }

    /// Create a sample coordination request for testing
    #[wasm_bindgen(js_name = createSampleRequest)]
    pub fn create_sample_request() -> String {
        let request = CoordinationRequestJs {
            tool_name: "test_tool".to_string(),
            tool_args: serde_json::json!({
                "query": "sample query",
                "param": "sample parameter"
            }),
            environment: {
                let mut env = HashMap::new();
                env.insert("NODE_ENV".to_string(), "development".to_string());
                env.insert("CONSCIOUSNESS_MODE".to_string(), "active".to_string());
                env
            },
            project_path: Some("./sample-project".to_string()),
            active_perceptions: vec!["human-insight".to_string(), "ai-analysis".to_string()],
            paradox_tolerance: 0.5,
        };

        serde_json::to_string(&request).unwrap_or_else(|_| "{}".to_string())
    }

    /// Get version information
    #[wasm_bindgen(js_name = getVersion)]
    pub fn get_version() -> String {
        serde_json::json!({
            "version": env!("CARGO_PKG_VERSION"),
            "name": "context-casial-xpress",
            "part_of": "ubiquity-os",
            "consciousness_aware": true,
            "paradox_resilient": true,
            "hydraulic_lime_principle": "stronger_under_pressure",
            "wasm_target": "universal_substrate"
        })
        .to_string()
    }

    /// Validate JSON structure for mission configuration
    #[wasm_bindgen(js_name = validateMissionJson)]
    pub fn validate_mission_json(json_str: &str) -> Result<String, JsValue> {
        let _mission: CasialMission = serde_json::from_str(json_str)
            .map_err(|e| JsValue::from_str(&format!("Invalid mission JSON: {}", e)))?;

        Ok(serde_json::json!({
            "valid": true,
            "message": "Mission configuration is valid"
        })
        .to_string())
    }

    /// Log message to browser console (for debugging)
    #[wasm_bindgen(js_name = logMessage)]
    pub fn log_message(level: &str, message: &str) {
        match level {
            "error" => web_sys::console::error_1(&JsValue::from_str(message)),
            "warn" => web_sys::console::warn_1(&JsValue::from_str(message)),
            "info" => web_sys::console::info_1(&JsValue::from_str(message)),
            _ => web_sys::console::log_1(&JsValue::from_str(message)),
        }
    }
}

/// TypeScript definitions for better JavaScript integration
#[wasm_bindgen(typescript_custom_section)]
const TYPESCRIPT_DEFINITIONS: &'static str = r#"
export interface CoordinationRequestJs {
    tool_name: string;
    tool_args: any;
    environment: Record<string, string>;
    project_path?: string;
    active_perceptions: string[];
    paradox_tolerance: number;
}

export interface CoordinationResultJs {
    applied: boolean;
    injected_content: string;
    modified_args: any;
    activated_rules: string[];
    used_templates: string[];
    paradoxes_detected: ParadoxReportJs[];
    metadata: Record<string, any>;
}

export interface ParadoxReportJs {
    id: string;
    description: string;
    severity: string;
    resolution_strategy: string;
    confidence_impact: number;
}

export class CasialEngineWasm {
    constructor();
    loadMissionFromJson(mission_json: string): void;
    coordinate(request_json: string): string;
    getCoordinationHistory(): string;
    getParadoxRegistry(): string;
    getStatistics(): string;
}

export class CasialUtils {
    static createSampleMission(): string;
    static createSampleRequest(): string;
    static getVersion(): string;
    static validateMissionJson(json_str: string): string;
    static logMessage(level: string, message: string): void;
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_engine_creation() {
        let engine = CasialEngineWasm::new();
        let stats = engine.get_statistics();
        assert!(stats.contains("consciousness_aware"));
    }

    #[wasm_bindgen_test]
    fn test_sample_mission() {
        let mission_json = CasialUtils::create_sample_mission();
        assert!(mission_json.contains("sample-wasm-mission"));
    }

    #[wasm_bindgen_test]
    fn test_sample_request() {
        let request_json = CasialUtils::create_sample_request();
        assert!(request_json.contains("test_tool"));
    }

    #[wasm_bindgen_test]
    fn test_version_info() {
        let version = CasialUtils::get_version();
        assert!(version.contains("context-casial-xpress"));
        assert!(version.contains("ubiquity-os"));
    }
}
