//! # Paradox Module
//!
//! Handles contradictory information and conflicting perceptions.
//! The Casial system thrives on paradox - like hydraulic lime getting stronger under pressure.

use crate::{CasialError, ParadoxStrategy, PerceptionId};
use ahash::AHashMap;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A detected paradox in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paradox {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub conflicting_elements: Vec<ParadoxElement>,
    pub severity: ParadoxSeverity,
    pub resolution_strategy: ParadoxStrategy,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolution_outcome: Option<ParadoxResolution>,
    pub metadata: AHashMap<String, serde_json::Value>,
}

/// An element involved in a paradox
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParadoxElement {
    pub element_type: ParadoxElementType,
    pub element_id: String,
    pub confidence: f64,
    pub evidence: Vec<String>,
    pub perspective: Option<PerceptionId>,
}

/// Types of elements that can be involved in paradoxes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParadoxElementType {
    /// A template with specific content
    Template,
    /// A perception or viewpoint
    Perception,
    /// Environmental context
    Environment,
    /// Tool behavior or configuration
    Tool,
    /// Mission objective or rule
    Mission,
}

/// Severity levels for paradoxes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum ParadoxSeverity {
    /// Minor conflict, easily resolved
    Low,
    /// Moderate conflict requiring attention
    Medium,
    /// Significant conflict requiring intervention
    High,
    /// Critical conflict that blocks operation
    Critical,
}

/// The outcome of paradox resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParadoxResolution {
    pub strategy_used: ParadoxStrategy,
    pub resolution_time_ms: f64,
    pub confidence_impact: f64,
    pub synthesis_result: Option<String>,
    pub chosen_elements: Vec<String>,
    pub metadata: AHashMap<String, serde_json::Value>,
}

/// Manager for detecting and resolving paradoxes
pub struct ParadoxManager {
    active_paradoxes: AHashMap<Uuid, Paradox>,
    resolved_paradoxes: AHashMap<Uuid, Paradox>,
    resolution_history: Vec<ParadoxResolutionEvent>,
    detection_rules: Vec<ParadoxDetectionRule>,
}

/// An event in the paradox resolution history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParadoxResolutionEvent {
    pub paradox_id: Uuid,
    pub event_type: ResolutionEventType,
    pub timestamp: DateTime<Utc>,
    pub details: serde_json::Value,
}

/// Types of paradox resolution events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResolutionEventType {
    Detected,
    AnalysisStarted,
    StrategySelected,
    ResolutionAttempted,
    Resolved,
    Escalated,
    Timeout,
}

/// Rules for detecting paradoxes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParadoxDetectionRule {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub detection_pattern: DetectionPattern,
    pub severity_threshold: ParadoxSeverity,
    pub auto_resolve: bool,
    pub preferred_strategy: ParadoxStrategy,
}

/// Patterns for detecting paradoxes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DetectionPattern {
    /// Conflicting template content
    ConflictingTemplates {
        similarity_threshold: f64,
        contradiction_keywords: Vec<String>,
    },
    /// Contradictory perceptions
    ConflictingPerceptions {
        confidence_threshold: f64,
        overlap_threshold: f64,
    },
    /// Inconsistent environmental signals
    EnvironmentalConflict {
        variable_patterns: Vec<String>,
        value_conflicts: Vec<(String, String)>,
    },
    /// Tool behavior conflicts
    ToolConflicts {
        tool_categories: Vec<String>,
        behavior_patterns: Vec<String>,
    },
}

impl ParadoxManager {
    /// Create a new paradox manager
    pub fn new() -> Self {
        let mut manager = Self {
            active_paradoxes: AHashMap::new(),
            resolved_paradoxes: AHashMap::new(),
            resolution_history: Vec::new(),
            detection_rules: Vec::new(),
        };

        // Add default detection rules
        manager.add_default_detection_rules();
        manager
    }

    /// Add default paradox detection rules
    fn add_default_detection_rules(&mut self) {
        let rules = vec![
            ParadoxDetectionRule {
                id: "conflicting-templates".to_string(),
                name: "Conflicting Template Content".to_string(),
                enabled: true,
                detection_pattern: DetectionPattern::ConflictingTemplates {
                    similarity_threshold: 0.7,
                    contradiction_keywords: vec![
                        "not".to_string(),
                        "never".to_string(),
                        "don't".to_string(),
                        "avoid".to_string(),
                        "opposite".to_string(),
                    ],
                },
                severity_threshold: ParadoxSeverity::Medium,
                auto_resolve: false,
                preferred_strategy: ParadoxStrategy::Coexist,
            },
            ParadoxDetectionRule {
                id: "perception-conflicts".to_string(),
                name: "Conflicting Perceptions".to_string(),
                enabled: true,
                detection_pattern: DetectionPattern::ConflictingPerceptions {
                    confidence_threshold: 0.8,
                    overlap_threshold: 0.5,
                },
                severity_threshold: ParadoxSeverity::High,
                auto_resolve: true,
                preferred_strategy: ParadoxStrategy::Synthesize,
            },
        ];

        self.detection_rules.extend(rules);
    }

    /// Detect paradoxes in the given context
    pub fn detect_paradoxes(
        &mut self,
        templates: &[crate::CasialTemplate],
        perceptions: &[crate::Perception],
        environment: &AHashMap<String, String>,
    ) -> Result<Vec<Uuid>> {
        let mut detected_paradoxes = Vec::new();

        for rule in &self.detection_rules {
            if !rule.enabled {
                continue;
            }

            let paradoxes = self.apply_detection_rule(rule, templates, perceptions, environment)?;
            for paradox in paradoxes {
                let paradox_id = paradox.id;
                self.active_paradoxes.insert(paradox_id, paradox);
                detected_paradoxes.push(paradox_id);

                // Record detection event
                self.resolution_history.push(ParadoxResolutionEvent {
                    paradox_id,
                    event_type: ResolutionEventType::Detected,
                    timestamp: Utc::now(),
                    details: serde_json::json!({
                        "rule_id": rule.id,
                        "rule_name": rule.name
                    }),
                });
            }
        }

        Ok(detected_paradoxes)
    }

    /// Apply a specific detection rule
    fn apply_detection_rule(
        &self,
        rule: &ParadoxDetectionRule,
        templates: &[crate::CasialTemplate],
        perceptions: &[crate::Perception],
        environment: &AHashMap<String, String>,
    ) -> Result<Vec<Paradox>> {
        let mut paradoxes = Vec::new();

        match &rule.detection_pattern {
            DetectionPattern::ConflictingTemplates {
                similarity_threshold,
                contradiction_keywords,
            } => {
                paradoxes.extend(self.detect_template_conflicts(
                    templates,
                    *similarity_threshold,
                    contradiction_keywords,
                    &rule.preferred_strategy,
                )?);
            }
            DetectionPattern::ConflictingPerceptions {
                confidence_threshold,
                overlap_threshold,
            } => {
                paradoxes.extend(self.detect_perception_conflicts(
                    perceptions,
                    *confidence_threshold,
                    *overlap_threshold,
                    &rule.preferred_strategy,
                )?);
            }
            DetectionPattern::EnvironmentalConflict {
                variable_patterns,
                value_conflicts,
            } => {
                paradoxes.extend(self.detect_environmental_conflicts(
                    environment,
                    variable_patterns,
                    value_conflicts,
                    &rule.preferred_strategy,
                )?);
            }
            DetectionPattern::ToolConflicts {
                tool_categories: _,
                behavior_patterns: _,
            } => {
                // Tool conflict detection would be implemented here
                // For now, we'll skip this as it requires more context
            }
        }

        Ok(paradoxes)
    }

    /// Detect conflicts between templates
    fn detect_template_conflicts(
        &self,
        templates: &[crate::CasialTemplate],
        similarity_threshold: f64,
        contradiction_keywords: &[String],
        strategy: &ParadoxStrategy,
    ) -> Result<Vec<Paradox>> {
        let mut conflicts = Vec::new();

        for i in 0..templates.len() {
            for j in (i + 1)..templates.len() {
                let template_a = &templates[i];
                let template_b = &templates[j];

                // Check for contradictory keywords
                let has_contradiction = contradiction_keywords.iter().any(|keyword| {
                    (template_a.content.contains(keyword) && !template_b.content.contains(keyword))
                        || (!template_a.content.contains(keyword)
                            && template_b.content.contains(keyword))
                });

                // Simple similarity check (in practice, use more sophisticated methods)
                let similarity =
                    self.calculate_content_similarity(&template_a.content, &template_b.content);

                if has_contradiction && similarity > similarity_threshold {
                    let paradox = Paradox {
                        id: Uuid::new_v4(),
                        name: format!("Template Conflict: {} vs {}", template_a.name, template_b.name),
                        description: format!(
                            "Templates '{}' and '{}' contain contradictory guidance with high content similarity",
                            template_a.name, template_b.name
                        ),
                        conflicting_elements: vec![
                            ParadoxElement {
                                element_type: ParadoxElementType::Template,
                                element_id: template_a.id.clone(),
                                confidence: 1.0 - template_a.paradox_resistance,
                                evidence: vec![template_a.content.clone()],
                                perspective: template_a.perception_affinity.first().copied(),
                            },
                            ParadoxElement {
                                element_type: ParadoxElementType::Template,
                                element_id: template_b.id.clone(),
                                confidence: 1.0 - template_b.paradox_resistance,
                                evidence: vec![template_b.content.clone()],
                                perspective: template_b.perception_affinity.first().copied(),
                            },
                        ],
                        severity: if similarity > 0.9 {
                            ParadoxSeverity::High
                        } else {
                            ParadoxSeverity::Medium
                        },
                        resolution_strategy: strategy.clone(),
                        created_at: Utc::now(),
                        resolved_at: None,
                        resolution_outcome: None,
                        metadata: AHashMap::from([
                            ("similarity".to_string(), serde_json::json!(similarity)),
                            ("contradiction_detected".to_string(), serde_json::json!(has_contradiction)),
                        ]),
                    };

                    conflicts.push(paradox);
                }
            }
        }

        Ok(conflicts)
    }

    /// Detect conflicts between perceptions
    fn detect_perception_conflicts(
        &self,
        perceptions: &[crate::Perception],
        confidence_threshold: f64,
        overlap_threshold: f64,
        strategy: &ParadoxStrategy,
    ) -> Result<Vec<Paradox>> {
        let mut conflicts = Vec::new();

        for i in 0..perceptions.len() {
            for j in (i + 1)..perceptions.len() {
                let perception_a = &perceptions[i];
                let perception_b = &perceptions[j];

                if perception_a.confidence < confidence_threshold
                    || perception_b.confidence < confidence_threshold
                {
                    continue;
                }

                // Check for conceptual overlap (simplified)
                let overlap = self.calculate_perception_overlap(perception_a, perception_b);

                if overlap > overlap_threshold {
                    let paradox = Paradox {
                        id: Uuid::new_v4(),
                        name: format!("Perception Conflict: {} vs {}", perception_a.name, perception_b.name),
                        description: format!(
                            "High-confidence perceptions '{}' and '{}' have overlapping domains but different conclusions",
                            perception_a.name, perception_b.name
                        ),
                        conflicting_elements: vec![
                            ParadoxElement {
                                element_type: ParadoxElementType::Perception,
                                element_id: perception_a.id.0.to_string(),
                                confidence: perception_a.confidence,
                                evidence: vec![perception_a.description.clone()],
                                perspective: Some(perception_a.id),
                            },
                            ParadoxElement {
                                element_type: ParadoxElementType::Perception,
                                element_id: perception_b.id.0.to_string(),
                                confidence: perception_b.confidence,
                                evidence: vec![perception_b.description.clone()],
                                perspective: Some(perception_b.id),
                            },
                        ],
                        severity: if overlap > 0.8 {
                            ParadoxSeverity::Critical
                        } else {
                            ParadoxSeverity::High
                        },
                        resolution_strategy: strategy.clone(),
                        created_at: Utc::now(),
                        resolved_at: None,
                        resolution_outcome: None,
                        metadata: AHashMap::from([
                            ("overlap_score".to_string(), serde_json::json!(overlap)),
                        ]),
                    };

                    conflicts.push(paradox);
                }
            }
        }

        Ok(conflicts)
    }

    /// Detect environmental conflicts
    fn detect_environmental_conflicts(
        &self,
        environment: &AHashMap<String, String>,
        variable_patterns: &[String],
        value_conflicts: &[(String, String)],
        strategy: &ParadoxStrategy,
    ) -> Result<Vec<Paradox>> {
        let mut conflicts = Vec::new();

        // Check for conflicting environment variables
        for (conflict_a, conflict_b) in value_conflicts {
            for pattern in variable_patterns {
                if let Some(value) = environment.get(pattern) {
                    if (value.contains(conflict_a) && value.contains(conflict_b))
                        || (environment.values().any(|v| v.contains(conflict_a))
                            && environment.values().any(|v| v.contains(conflict_b)))
                    {
                        let paradox = Paradox {
                            id: Uuid::new_v4(),
                            name: "Environmental Conflict".to_string(),
                            description: format!(
                                "Environment contains conflicting values: '{}' and '{}'",
                                conflict_a, conflict_b
                            ),
                            conflicting_elements: vec![
                                ParadoxElement {
                                    element_type: ParadoxElementType::Environment,
                                    element_id: conflict_a.clone(),
                                    confidence: 1.0,
                                    evidence: vec![format!(
                                        "Pattern: {}, Value: {}",
                                        pattern, value
                                    )],
                                    perspective: None,
                                },
                                ParadoxElement {
                                    element_type: ParadoxElementType::Environment,
                                    element_id: conflict_b.clone(),
                                    confidence: 1.0,
                                    evidence: vec![format!(
                                        "Pattern: {}, Value: {}",
                                        pattern, value
                                    )],
                                    perspective: None,
                                },
                            ],
                            severity: ParadoxSeverity::Medium,
                            resolution_strategy: strategy.clone(),
                            created_at: Utc::now(),
                            resolved_at: None,
                            resolution_outcome: None,
                            metadata: AHashMap::from([
                                ("pattern".to_string(), serde_json::json!(pattern)),
                                ("detected_value".to_string(), serde_json::json!(value)),
                            ]),
                        };

                        conflicts.push(paradox);
                    }
                }
            }
        }

        Ok(conflicts)
    }

    /// Resolve a detected paradox
    pub fn resolve_paradox(&mut self, paradox_id: Uuid) -> Result<ParadoxResolution> {
        // First get the paradox immutably to extract needed data
        let (strategy, conflicting_elements, description) = {
            let paradox = self.active_paradoxes.get(&paradox_id).ok_or_else(|| {
                CasialError::ParadoxTimeout(format!("Paradox {} not found", paradox_id))
            })?;
            (paradox.resolution_strategy.clone(), paradox.conflicting_elements.clone(), paradox.description.clone())
        };

        let start_time = std::time::Instant::now();

        let resolution = match &strategy {
            ParadoxStrategy::Ignore => {
                // Simply remove the paradox without resolution
                ParadoxResolution {
                    strategy_used: ParadoxStrategy::Ignore,
                    resolution_time_ms: start_time.elapsed().as_secs_f64() * 1000.0,
                    confidence_impact: 0.0,
                    synthesis_result: None,
                    chosen_elements: vec![],
                    metadata: AHashMap::new(),
                }
            }
            ParadoxStrategy::Coexist => {
                // Keep all conflicting elements
                ParadoxResolution {
                    strategy_used: ParadoxStrategy::Coexist,
                    resolution_time_ms: start_time.elapsed().as_secs_f64() * 1000.0,
                    confidence_impact: -0.1, // Slight confidence reduction
                    synthesis_result: Some("Multiple perspectives maintained".to_string()),
                    chosen_elements: conflicting_elements
                        .iter()
                        .map(|e| e.element_id.clone())
                        .collect(),
                    metadata: AHashMap::new(),
                }
            }
            ParadoxStrategy::Synthesize => {
                // Attempt to create a higher-order synthesis
                let synthesis = self.synthesize_paradox_elements(&conflicting_elements);
                ParadoxResolution {
                    strategy_used: ParadoxStrategy::Synthesize,
                    resolution_time_ms: start_time.elapsed().as_secs_f64() * 1000.0,
                    confidence_impact: 0.1, // Synthesis can increase confidence
                    synthesis_result: Some(synthesis),
                    chosen_elements: vec!["synthesized".to_string()],
                    metadata: AHashMap::new(),
                }
            }
            ParadoxStrategy::Expose => {
                // Make the paradox explicit
                ParadoxResolution {
                    strategy_used: ParadoxStrategy::Expose,
                    resolution_time_ms: start_time.elapsed().as_secs_f64() * 1000.0,
                    confidence_impact: 0.0,
                    synthesis_result: Some(format!("PARADOX DETECTED: {}", description)),
                    chosen_elements: conflicting_elements
                        .iter()
                        .map(|e| e.element_id.clone())
                        .collect(),
                    metadata: AHashMap::new(),
                }
            }
        };

        // Now get mutable access to update the paradox
        let paradox = self.active_paradoxes.get_mut(&paradox_id).unwrap(); // We know it exists

        // Mark as resolved
        paradox.resolved_at = Some(Utc::now());
        paradox.resolution_outcome = Some(resolution.clone());

        // Move to resolved paradoxes
        let resolved_paradox = paradox.clone();
        self.resolved_paradoxes.insert(paradox_id, resolved_paradox);
        self.active_paradoxes.remove(&paradox_id);

        // Record resolution event
        self.resolution_history.push(ParadoxResolutionEvent {
            paradox_id,
            event_type: ResolutionEventType::Resolved,
            timestamp: Utc::now(),
            details: serde_json::json!({
                "strategy": resolution.strategy_used,
                "resolution_time_ms": resolution.resolution_time_ms
            }),
        });

        Ok(resolution)
    }

    /// Synthesize conflicting elements into a higher-order understanding
    fn synthesize_paradox_elements(&self, conflicting_elements: &[ParadoxElement]) -> String {
        // This is a simplified synthesis algorithm
        // In practice, this would use more sophisticated techniques
        match conflicting_elements.len() {
            2 => {
                let element_a = &conflicting_elements[0];
                let element_b = &conflicting_elements[1];

                format!(
                    "SYNTHESIS: Both '{}' and '{}' perspectives have validity. \
                    Consider '{}' in contexts requiring {} confidence, \
                    and '{}' where {} confidence is appropriate. \
                    The apparent contradiction may reflect different operational contexts.",
                    element_a.element_id,
                    element_b.element_id,
                    element_a.element_id,
                    element_a.confidence,
                    element_b.element_id,
                    element_b.confidence
                )
            }
            _ => {
                format!(
                    "SYNTHESIS: Multiple conflicting perspectives detected ({}). \
                    Consider contextual application based on specific circumstances \
                    and confidence levels of each perspective.",
                    conflicting_elements.len()
                )
            }
        }
    }

    /// Calculate content similarity between two strings
    fn calculate_content_similarity(&self, content_a: &str, content_b: &str) -> f64 {
        // Simplified similarity calculation
        // In practice, use more sophisticated methods like cosine similarity
        let words_a: std::collections::HashSet<&str> = content_a.split_whitespace().collect();
        let words_b: std::collections::HashSet<&str> = content_b.split_whitespace().collect();

        let intersection = words_a.intersection(&words_b).count();
        let union = words_a.union(&words_b).count();

        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }

    /// Calculate overlap between two perceptions
    fn calculate_perception_overlap(
        &self,
        perception_a: &crate::Perception,
        perception_b: &crate::Perception,
    ) -> f64 {
        // Simplified overlap calculation based on description similarity
        self.calculate_content_similarity(&perception_a.description, &perception_b.description)
    }

    /// Get statistics about paradox detection and resolution
    pub fn get_statistics(&self) -> ParadoxManagerStats {
        let active_count = self.active_paradoxes.len();
        let resolved_count = self.resolved_paradoxes.len();
        let total_count = active_count + resolved_count;

        let avg_resolution_time = if resolved_count > 0 {
            self.resolved_paradoxes
                .values()
                .filter_map(|p| p.resolution_outcome.as_ref())
                .map(|r| r.resolution_time_ms)
                .sum::<f64>()
                / resolved_count as f64
        } else {
            0.0
        };

        let strategy_distribution: AHashMap<String, usize> = self
            .resolved_paradoxes
            .values()
            .filter_map(|p| p.resolution_outcome.as_ref())
            .fold(AHashMap::new(), |mut acc, outcome| {
                let strategy_name = format!("{:?}", outcome.strategy_used);
                *acc.entry(strategy_name).or_insert(0) += 1;
                acc
            });

        ParadoxManagerStats {
            active_paradoxes: active_count,
            resolved_paradoxes: resolved_count,
            total_paradoxes: total_count,
            average_resolution_time_ms: avg_resolution_time,
            strategy_distribution,
        }
    }
}

/// Statistics for paradox manager monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParadoxManagerStats {
    pub active_paradoxes: usize,
    pub resolved_paradoxes: usize,
    pub total_paradoxes: usize,
    pub average_resolution_time_ms: f64,
    pub strategy_distribution: AHashMap<String, usize>,
}

impl Default for ParadoxManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paradox_creation() {
        let paradox = Paradox {
            id: Uuid::new_v4(),
            name: "Test Paradox".to_string(),
            description: "A test paradox".to_string(),
            conflicting_elements: vec![],
            severity: ParadoxSeverity::Low,
            resolution_strategy: ParadoxStrategy::Ignore,
            created_at: Utc::now(),
            resolved_at: None,
            resolution_outcome: None,
            metadata: AHashMap::new(),
        };

        assert_eq!(paradox.severity, ParadoxSeverity::Low);
        assert!(paradox.resolved_at.is_none());
    }

    #[test]
    fn test_paradox_manager() {
        let manager = ParadoxManager::new();
        assert_eq!(manager.active_paradoxes.len(), 0);
        assert_eq!(manager.resolved_paradoxes.len(), 0);
        assert!(!manager.detection_rules.is_empty()); // Default rules should be added
    }

    #[test]
    fn test_content_similarity() {
        let manager = ParadoxManager::new();
        let similarity = manager.calculate_content_similarity("hello world", "hello universe");
        assert!(similarity > 0.0);
        assert!(similarity < 1.0);
    }
}
