//! # Mission Management
//!
//! Mission loading, validation, and management for consciousness-aware context coordination.

use anyhow::{Context, Result};
use casial_core::{CasialMission, CasialTemplate};
use std::{collections::HashMap, path::Path};

/// Mission manager for handling multiple missions
pub struct MissionManager {
    missions: HashMap<String, CasialMission>,
}

impl MissionManager {
    pub fn new() -> Self {
        Self {
            missions: HashMap::new(),
        }
    }

    pub fn add_mission(&mut self, mission: CasialMission) -> Result<()> {
        let mission_id = mission.id.clone();
        self.missions.insert(mission_id, mission);
        Ok(())
    }

    pub fn get_mission(&self, id: &str) -> Option<&CasialMission> {
        self.missions.get(id)
    }

    pub fn get_all_missions(&self) -> Vec<&CasialMission> {
        self.missions.values().collect()
    }

    pub fn remove_mission(&mut self, id: &str) -> Option<CasialMission> {
        self.missions.remove(id)
    }
}

impl Default for MissionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Load mission from YAML file
pub fn load_mission_from_file<P: AsRef<Path>>(path: P) -> Result<CasialMission> {
    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read mission file: {}", path.as_ref().display()))?;

    let mission: CasialMission = serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse mission YAML: {}", path.as_ref().display()))?;

    Ok(mission)
}

/// Merge templates from project templates/ directory with front-matter parsing
pub fn merge_templates_from_dir(
    mission: &mut CasialMission,
    project_root: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;

    let templates_dir = std::path::Path::new(project_root).join("templates");
    if !templates_dir.exists() {
        tracing::debug!("No templates directory found at {:?}", templates_dir);
        return Ok(());
    }

    let entries = match fs::read_dir(&templates_dir) {
        Ok(entries) => entries,
        Err(e) => {
            tracing::warn!("Failed to read templates directory: {}", e);
            return Ok(());
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        // Only process common template file extensions
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        if !matches!(ext, "md" | "txt" | "yaml" | "yml" | "template") {
            continue;
        }

        let content = match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(e) => {
                tracing::warn!("Failed to read template file {:?}: {}", path, e);
                continue;
            }
        };

        let stem = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        // Parse front-matter if present
        let (template_id, template_name, template_content, categories, priority) =
            if content.starts_with("---") {
                parse_front_matter(&content, &stem)
            } else {
                (
                    stem.clone(),
                    stem.clone(),
                    content,
                    vec!["project".to_string()],
                    50,
                )
            };

        let template = CasialTemplate {
            id: template_id.clone(),
            name: template_name,
            description: format!("Loaded from project template: {}", path.display()),
            categories,
            priority: priority.try_into().unwrap(),
            enabled: true,
            content: template_content,
            perception_affinity: vec![], // Can be set in front-matter
            paradox_resistance: 0.7,     // Default resistance
            metadata: ahash::AHashMap::new(),
        };

        mission.templates.push(template);
        tracing::info!("Loaded project template: {}", template_id);
    }

    Ok(())
}

/// Parse YAML front-matter from template content
fn parse_front_matter(
    content: &str,
    fallback_name: &str,
) -> (String, String, String, Vec<String>, i32) {
    // Find end of front-matter block
    let front_matter_end = content[3..].find("\n---");

    if let Some(end_pos) = front_matter_end {
        let front_matter = &content[3..3 + end_pos];
        let body = &content[3 + end_pos + 4..].trim_start();

        // Parse YAML front-matter
        match serde_yaml::from_str::<serde_yaml::Value>(front_matter) {
            Ok(metadata) => {
                let id = metadata
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or(fallback_name)
                    .to_string();

                let name = metadata
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&id)
                    .to_string();

                let categories = metadata
                    .get("categories")
                    .and_then(|v| v.as_sequence())
                    .map(|seq| {
                        seq.iter()
                            .filter_map(|v| v.as_str())
                            .map(|s| s.to_string())
                            .collect()
                    })
                    .unwrap_or_else(|| vec!["project".to_string()]);

                let priority = metadata
                    .get("priority")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(50) as i32;

                (id, name, body.to_string(), categories, priority)
            }
            Err(e) => {
                tracing::warn!("Failed to parse front-matter YAML: {}", e);
                (
                    fallback_name.to_string(),
                    fallback_name.to_string(),
                    content.to_string(),
                    vec!["project".to_string()],
                    50,
                )
            }
        }
    } else {
        // No proper front-matter block found
        (
            fallback_name.to_string(),
            fallback_name.to_string(),
            content.to_string(),
            vec!["project".to_string()],
            50,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_mission_manager() {
        let mut manager = MissionManager::new();
        assert_eq!(manager.get_all_missions().len(), 0);
    }

    #[test]
    fn test_load_mission_from_file() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(
            temp_file,
            r#"
id: test-mission
name: Test Mission
description: A test mission
templates: []
rules: []
perceptions: []
budgets:
  global_char_limit: 1000
  per_tool_limits: {{}}
  perception_quotas: {{}}
  paradox_overhead: 0.1
created_at: "2025-01-01T00:00:00Z"
updated_at: "2025-01-01T00:00:00Z"
"#
        )?;

        let mission = load_mission_from_file(temp_file.path())?;
        assert_eq!(mission.id, "test-mission");
        assert_eq!(mission.name, "Test Mission");

        Ok(())
    }
}
