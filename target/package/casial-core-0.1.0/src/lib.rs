//! # Casial Core
//!
//! The consciousness-computation substrate for context coordination.
//!
//! Named after Smeaton's hydraulic lime (casial) - a material that grows stronger
//! under pressure and adversarial conditions. This engine handles paradoxes and
//! contradictory perceptions that traditional systems cannot recognize exist.

use ahash::AHashMap;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

pub mod coordination;
pub mod paradox;
pub mod perception;
pub mod substrate;

// Re-exports for convenience
pub use coordination::*;
pub use paradox::*;
pub use perception::*;
pub use substrate::*;

/// Core errors in the Casial system
#[derive(thiserror::Error, Debug)]
pub enum CasialError {
    #[error("Perception lock failed: {0}")]
    PerceptionLock(String),

    #[error("Paradox resolution timeout: {0}")]
    ParadoxTimeout(String),

    #[error("Context coordination failed: {0}")]
    CoordinationFailure(String),

    #[error("Template processing error: {0}")]
    TemplateError(String),

    #[error("Mission configuration error: {0}")]
    MissionError(String),

    #[error("Substrate integration error: {0}")]
    SubstrateError(String),
}

/// A unique identifier for perception states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PerceptionId(Uuid);

impl PerceptionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for PerceptionId {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents different ways of seeing reality that can coexist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Perception {
    pub id: PerceptionId,
    pub name: String,
    pub description: String,
    pub confidence: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: AHashMap<String, serde_json::Value>,
}

/// A template that can be applied to context injection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CasialTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub categories: Vec<String>,
    pub priority: u32,
    pub enabled: bool,
    pub content: String,
    pub perception_affinity: Vec<PerceptionId>,
    pub paradox_resistance: f64, // How well it handles contradictory contexts
    pub metadata: AHashMap<String, serde_json::Value>,
}

/// Rules for when and how to apply templates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationRule {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub conditions: RuleConditions,
    pub actions: RuleActions,
    pub perception_scope: Vec<PerceptionId>,
    pub paradox_handling: ParadoxStrategy,
}

/// Conditions that must be met for a rule to activate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleConditions {
    pub tool_patterns: Vec<String>,
    pub environment_vars: AHashMap<String, String>,
    pub file_signals: Vec<FileSignal>,
    pub perception_states: Vec<PerceptionId>,
    pub min_confidence: Option<f64>,
}

/// Actions to take when a rule activates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleActions {
    pub template_ids: Vec<String>,
    pub transform_type: TransformType,
    pub target_field: Option<String>,
    pub char_limit: Option<usize>,
    pub perception_lock: bool,
}

/// File system signals for contextual awareness
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSignal {
    pub path: String,
    pub must_exist: bool,
    pub contains: Option<String>,
    pub modified_since: Option<DateTime<Utc>>,
}

/// How to transform the injected content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransformType {
    Prepend,
    Append,
    InjectField,
    SystemInstruction,
    PerceptionLayer,
}

/// Strategy for handling paradoxes (contradictory information)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParadoxStrategy {
    /// Ignore contradictions (traditional approach)
    Ignore,
    /// Acknowledge both perspectives without resolution
    Coexist,
    /// Attempt to synthesize a higher-order understanding
    Synthesize,
    /// Present the paradox explicitly to the user
    Expose,
}

/// A mission defines the overall context coordination strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CasialMission {
    pub id: String,
    pub name: String,
    pub description: String,
    pub templates: Vec<CasialTemplate>,
    pub rules: Vec<CoordinationRule>,
    pub perceptions: Vec<Perception>,
    pub budgets: BudgetConfiguration,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Budget configuration for resource management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetConfiguration {
    pub global_char_limit: Option<usize>,
    pub per_tool_limits: AHashMap<String, usize>,
    pub perception_quotas: AHashMap<PerceptionId, usize>,
    pub paradox_overhead: f64, // Additional resources for paradox handling
}

/// Input for context coordination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationRequest {
    pub tool_name: String,
    pub tool_args: serde_json::Value,
    pub environment: AHashMap<String, String>,
    pub project_path: Option<String>,
    pub active_perceptions: Vec<PerceptionId>,
    pub paradox_tolerance: f64,
}

/// Result of context coordination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationResult {
    pub applied: bool,
    pub injected_content: String,
    pub modified_args: serde_json::Value,
    pub activated_rules: Vec<String>,
    pub used_templates: Vec<String>,
    pub perception_locks: Vec<PerceptionId>,
    pub paradoxes_detected: Vec<ParadoxReport>,
    pub metadata: AHashMap<String, serde_json::Value>,
}

/// Report of paradox detection and handling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParadoxReport {
    pub id: Uuid,
    pub description: String,
    pub conflicting_perceptions: Vec<PerceptionId>,
    pub resolution_strategy: ParadoxStrategy,
    pub confidence_impact: f64,
}

/// The main Casial coordination engine
pub struct CasialEngine {
    missions: Arc<DashMap<String, Arc<CasialMission>>>,
    active_perceptions: Arc<DashMap<PerceptionId, Arc<RwLock<Perception>>>>,
    coordination_history: Arc<DashMap<Uuid, CoordinationResult>>,
    paradox_registry: Arc<DashMap<Uuid, ParadoxReport>>,
}

impl CasialEngine {
    /// Create a new Casial engine
    pub fn new() -> Self {
        Self {
            missions: Arc::new(DashMap::new()),
            active_perceptions: Arc::new(DashMap::new()),
            coordination_history: Arc::new(DashMap::new()),
            paradox_registry: Arc::new(DashMap::new()),
        }
    }

    /// Load a mission into the engine
    pub fn load_mission(&self, mission: CasialMission) -> Result<()> {
        let mission_id = mission.id.clone();
        let mission_arc = Arc::new(mission);

        // Register perceptions from this mission
        for perception in &mission_arc.perceptions {
            self.active_perceptions
                .insert(perception.id, Arc::new(RwLock::new(perception.clone())));
        }

        self.missions.insert(mission_id, mission_arc);
        Ok(())
    }

    /// Coordinate context for a tool request
    pub fn coordinate(&self, request: CoordinationRequest) -> Result<CoordinationResult> {
        // Find applicable missions (could be multiple for different perceptions)
        let applicable_missions: Vec<Arc<CasialMission>> = self
            .missions
            .iter()
            .map(|entry| entry.value().clone())
            .collect();

        if applicable_missions.is_empty() {
            return Ok(CoordinationResult {
                applied: false,
                injected_content: String::new(),
                modified_args: request.tool_args,
                activated_rules: vec![],
                used_templates: vec![],
                perception_locks: vec![],
                paradoxes_detected: vec![],
                metadata: AHashMap::new(),
            });
        }

        // Evaluate rules across all applicable missions
        let mut activated_rules = Vec::new();
        let mut applicable_templates = AHashMap::new();
        let mut detected_paradoxes = Vec::new();

        for mission in &applicable_missions {
            for rule in &mission.rules {
                if !rule.enabled {
                    continue;
                }

                if self.evaluate_rule_conditions(&rule.conditions, &request)? {
                    activated_rules.push(rule.id.clone());

                    // Collect templates from this rule
                    for template_id in &rule.actions.template_ids {
                        if let Some(template) =
                            mission.templates.iter().find(|t| t.id == *template_id)
                        {
                            // Check for perception conflicts (paradoxes)
                            if let Some(existing) = applicable_templates.get(template_id) {
                                let existing_template: &CasialTemplate = existing;
                                if !existing_template.perception_affinity.is_empty()
                                    && !template.perception_affinity.is_empty()
                                    && existing_template.perception_affinity
                                        != template.perception_affinity
                                {
                                    // Paradox detected!
                                    let paradox = ParadoxReport {
                                        id: Uuid::new_v4(),
                                        description: format!(
                                            "Template '{}' has conflicting perception affinities",
                                            template_id
                                        ),
                                        conflicting_perceptions: [
                                            existing_template.perception_affinity.clone(),
                                            template.perception_affinity.clone(),
                                        ]
                                        .concat(),
                                        resolution_strategy: rule.paradox_handling.clone(),
                                        confidence_impact: 1.0 - template.paradox_resistance,
                                    };

                                    detected_paradoxes.push(paradox.clone());
                                    self.paradox_registry.insert(paradox.id, paradox);
                                }
                            }

                            applicable_templates.insert(template_id.clone(), template.clone());
                        }
                    }
                }
            }
        }

        // Apply paradox handling strategies
        let resolved_templates = self.resolve_paradoxes(
            applicable_templates,
            &detected_paradoxes,
            request.paradox_tolerance,
        )?;

        // Compose final content
        let (injected_content, used_templates) =
            self.compose_context(resolved_templates, &applicable_missions[0].budgets)?;

        // Apply transformations
        let modified_args = self.apply_transformation(
            &request.tool_args,
            &injected_content,
            &activated_rules,
            &applicable_missions,
        )?;

        let result = CoordinationResult {
            applied: !used_templates.is_empty(),
            injected_content,
            modified_args,
            activated_rules,
            used_templates,
            perception_locks: request.active_perceptions.clone(),
            paradoxes_detected: detected_paradoxes,
            metadata: self.generate_metadata(&request)?,
        };

        // Store in history
        let history_id = Uuid::new_v4();
        self.coordination_history.insert(history_id, result.clone());

        Ok(result)
    }

    /// Evaluate if rule conditions are met
    fn evaluate_rule_conditions(
        &self,
        conditions: &RuleConditions,
        request: &CoordinationRequest,
    ) -> Result<bool> {
        // Tool pattern matching
        if !conditions.tool_patterns.is_empty() {
            let matches = conditions
                .tool_patterns
                .iter()
                .any(|pattern| request.tool_name.contains(pattern));
            if !matches {
                return Ok(false);
            }
        }

        // Environment variable matching
        for (key, expected) in &conditions.environment_vars {
            if let Some(actual) = request.environment.get(key) {
                if !actual.contains(expected) {
                    return Ok(false);
                }
            } else {
                return Ok(false);
            }
        }

        // File signal evaluation
        if let Some(project_path) = &request.project_path {
            for signal in &conditions.file_signals {
                if !self.evaluate_file_signal(signal, project_path)? {
                    return Ok(false);
                }
            }
        }

        // Perception state matching
        if !conditions.perception_states.is_empty() {
            let has_required_perception = conditions
                .perception_states
                .iter()
                .any(|required| request.active_perceptions.contains(required));
            if !has_required_perception {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Evaluate a file signal condition
    fn evaluate_file_signal(&self, signal: &FileSignal, project_path: &str) -> Result<bool> {
        let file_path = std::path::Path::new(project_path).join(&signal.path);

        let exists = file_path.exists();
        if signal.must_exist && !exists {
            return Ok(false);
        }

        if exists {
            if let Some(expected_content) = &signal.contains {
                let content = std::fs::read_to_string(&file_path)
                    .context("Failed to read file for signal evaluation")?;
                if !content.contains(expected_content) {
                    return Ok(false);
                }
            }

            if let Some(modified_since) = signal.modified_since {
                let metadata =
                    std::fs::metadata(&file_path).context("Failed to read file metadata")?;
                let modified = metadata
                    .modified()
                    .context("Failed to get file modification time")?;
                let modified_dt = DateTime::<Utc>::from(modified);
                if modified_dt < modified_since {
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }

    /// Resolve paradoxes using various strategies
    fn resolve_paradoxes(
        &self,
        templates: AHashMap<String, CasialTemplate>,
        paradoxes: &[ParadoxReport],
        tolerance: f64,
    ) -> Result<Vec<CasialTemplate>> {
        let mut resolved = Vec::new();
        let mut processed_ids = std::collections::HashSet::new();

        for template in templates.into_values() {
            if processed_ids.contains(&template.id) {
                continue;
            }

            // Check if this template is involved in any paradoxes
            let involved_paradoxes: Vec<&ParadoxReport> = paradoxes
                .iter()
                .filter(|p| p.confidence_impact > tolerance)
                .collect();

            if involved_paradoxes.is_empty() || template.paradox_resistance >= tolerance {
                // No significant paradoxes or template is resistant enough
                processed_ids.insert(template.id.clone());
                resolved.push(template);
            } else {
                // Handle paradox based on strategy
                let mut should_include = false;
                for paradox in &involved_paradoxes {
                    match paradox.resolution_strategy {
                        ParadoxStrategy::Ignore => {
                            should_include = true;
                        }
                        ParadoxStrategy::Coexist => {
                            // Include template but mark the paradox
                            should_include = true;
                        }
                        ParadoxStrategy::Synthesize => {
                            // For now, include the most resistant template
                            if template.paradox_resistance >= 0.5 {
                                should_include = true;
                            }
                        }
                        ParadoxStrategy::Expose => {
                            // Include template and let the paradox be visible
                            should_include = true;
                        }
                    }
                }
                processed_ids.insert(template.id.clone());
                if should_include {
                    resolved.push(template);
                }
            }
        }

        Ok(resolved)
    }

    /// Compose context from resolved templates
    fn compose_context(
        &self,
        templates: Vec<CasialTemplate>,
        budget: &BudgetConfiguration,
    ) -> Result<(String, Vec<String>)> {
        let mut sorted_templates = templates;
        sorted_templates.sort_by_key(|t| t.priority);

        let mut content = String::new();
        let mut used_templates = Vec::new();
        let mut char_count = 0;

        let char_limit = budget.global_char_limit.unwrap_or(usize::MAX);
        let paradox_overhead = (char_limit as f64 * budget.paradox_overhead) as usize;
        let effective_limit = char_limit.saturating_sub(paradox_overhead);

        for template in sorted_templates {
            if !template.enabled {
                continue;
            }

            let template_content = format!("## {}\n\n{}\n\n", template.name, template.content);

            if char_count + template_content.len() > effective_limit {
                break;
            }

            content.push_str(&template_content);
            char_count += template_content.len();
            used_templates.push(template.id.clone());
        }

        Ok((content, used_templates))
    }

    /// Apply transformations to the tool arguments
    fn apply_transformation(
        &self,
        args: &serde_json::Value,
        content: &str,
        _rules: &[String],
        missions: &[Arc<CasialMission>],
    ) -> Result<serde_json::Value> {
        let mut modified_args = args.clone();

        // Find the primary transformation type (from the first applicable rule)
        let transform_type = missions
            .iter()
            .flat_map(|m| &m.rules)
            .find(|r| r.enabled)
            .map(|r| &r.actions.transform_type)
            .unwrap_or(&TransformType::Prepend);

        match transform_type {
            TransformType::Prepend => {
                if let Some(query) = modified_args.get_mut("query") {
                    if let Some(query_str) = query.as_str() {
                        *query = serde_json::Value::String(format!("{}\n\n{}", content, query_str));
                    }
                } else if let Some(instructions) = modified_args.get_mut("instructions") {
                    if let Some(instr_str) = instructions.as_str() {
                        *instructions =
                            serde_json::Value::String(format!("{}\n\n{}", content, instr_str));
                    }
                }
            }
            TransformType::Append => {
                if let Some(query) = modified_args.get_mut("query") {
                    if let Some(query_str) = query.as_str() {
                        *query = serde_json::Value::String(format!("{}\n\n{}", query_str, content));
                    }
                }
            }
            TransformType::InjectField => {
                if let Some(obj) = modified_args.as_object_mut() {
                    obj.insert(
                        "casial_context".to_string(),
                        serde_json::Value::String(content.to_string()),
                    );
                }
            }
            TransformType::SystemInstruction => {
                if let Some(obj) = modified_args.as_object_mut() {
                    obj.insert(
                        "system_context".to_string(),
                        serde_json::Value::String(content.to_string()),
                    );
                }
            }
            TransformType::PerceptionLayer => {
                if let Some(obj) = modified_args.as_object_mut() {
                    obj.insert(
                        "perception_context".to_string(),
                        serde_json::Value::String(content.to_string()),
                    );
                }
            }
        }

        Ok(modified_args)
    }

    /// Generate metadata for the coordination result
    fn generate_metadata(
        &self,
        request: &CoordinationRequest,
    ) -> Result<AHashMap<String, serde_json::Value>> {
        let mut metadata = AHashMap::new();

        metadata.insert(
            "timestamp".to_string(),
            serde_json::Value::String(Utc::now().to_rfc3339()),
        );
        metadata.insert(
            "tool_name".to_string(),
            serde_json::Value::String(request.tool_name.clone()),
        );
        metadata.insert(
            "perception_count".to_string(),
            serde_json::Value::Number(serde_json::Number::from(request.active_perceptions.len())),
        );
        metadata.insert(
            "paradox_tolerance".to_string(),
            serde_json::Value::Number(
                serde_json::Number::from_f64(request.paradox_tolerance).unwrap(),
            ),
        );

        Ok(metadata)
    }

    /// Get coordination history for analysis
    pub fn get_coordination_history(&self) -> Vec<CoordinationResult> {
        self.coordination_history
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get paradox registry for analysis
    pub fn get_paradox_registry(&self) -> Vec<ParadoxReport> {
        self.paradox_registry
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }
}

impl Default for CasialEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_casial_engine_creation() {
        let engine = CasialEngine::new();
        assert_eq!(engine.missions.len(), 0);
        assert_eq!(engine.active_perceptions.len(), 0);
    }

    #[test]
    fn test_perception_id_generation() {
        let id1 = PerceptionId::new();
        let id2 = PerceptionId::new();
        assert_ne!(id1, id2);
    }
}
