//! # Perception Module
//!
//! Handles different ways of seeing reality that can coexist without forcing consensus.
//! This is core to the consciousness-computation substrate.

use crate::{CasialError, PerceptionId};
use ahash::AHashMap;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// The confidence level of a perception
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct PerceptionConfidence(pub f64);

impl PerceptionConfidence {
    pub fn new(confidence: f64) -> Result<Self> {
        if !(0.0..=1.0).contains(&confidence) {
            return Err(CasialError::PerceptionLock(
                "Confidence must be between 0.0 and 1.0".to_string(),
            )
            .into());
        }
        Ok(Self(confidence))
    }

    pub fn value(&self) -> f64 {
        self.0
    }

    pub fn is_high(&self) -> bool {
        self.0 >= 0.8
    }

    pub fn is_low(&self) -> bool {
        self.0 <= 0.3
    }

    pub fn is_uncertain(&self) -> bool {
        (0.4..0.6).contains(&self.0)
    }
}

/// Different types of perceptions in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerceptionType {
    /// Human intuition and insight
    Human,
    /// AI analysis and reasoning
    Artificial,
    /// Hybrid human-AI collaboration
    Hybrid,
    /// System-generated perception from data patterns
    Systemic,
    /// External API or service perspective
    External,
}

/// A specific viewpoint or way of understanding reality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerceptionView {
    pub id: PerceptionId,
    pub name: String,
    pub description: String,
    pub perception_type: PerceptionType,
    pub confidence: PerceptionConfidence,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tags: Vec<String>,
    pub metadata: AHashMap<String, serde_json::Value>,

    /// Related perceptions that support this view
    pub supporting_perceptions: Vec<PerceptionId>,
    /// Perceptions that conflict with this view
    pub conflicting_perceptions: Vec<PerceptionId>,
    /// Evidence supporting this perception
    pub evidence: Vec<PerceptionEvidence>,
}

/// Evidence supporting or refuting a perception
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerceptionEvidence {
    pub id: uuid::Uuid,
    pub description: String,
    pub source: EvidenceSource,
    pub weight: f64, // How much this evidence affects confidence
    pub timestamp: DateTime<Utc>,
    pub data: serde_json::Value,
}

/// Source of perception evidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvidenceSource {
    /// Direct observation or measurement
    Observation,
    /// Logical reasoning or inference
    Reasoning,
    /// Historical data or patterns
    Historical,
    /// External validation or verification
    External,
    /// Collaborative consensus
    Consensus,
    /// Expert judgment
    Expert,
}

/// Manager for perception states and interactions
pub struct PerceptionManager {
    perceptions: AHashMap<PerceptionId, PerceptionView>,
    perception_relationships: AHashMap<PerceptionId, HashSet<PerceptionId>>,
    active_locks: AHashMap<PerceptionId, PerceptionLock>,
}

/// A lock on a perception to coordinate access
#[derive(Debug, Clone)]
pub struct PerceptionLock {
    pub perception_id: PerceptionId,
    pub locked_by: uuid::Uuid, // Session ID that holds the lock
    pub locked_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub lock_type: PerceptionLockType,
}

/// Types of perception locks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerceptionLockType {
    /// Exclusive read/write access
    Exclusive,
    /// Shared read access, blocks writes
    Shared,
    /// Advisory lock, doesn't block but signals intent
    Advisory,
}

impl PerceptionManager {
    /// Create a new perception manager
    pub fn new() -> Self {
        Self {
            perceptions: AHashMap::new(),
            perception_relationships: AHashMap::new(),
            active_locks: AHashMap::new(),
        }
    }

    /// Register a new perception
    pub fn register_perception(&mut self, perception: PerceptionView) -> Result<()> {
        let perception_id = perception.id;

        // Initialize relationships
        self.perception_relationships
            .insert(perception_id, HashSet::new());

        // Add relationships from this perception
        for supporting_id in &perception.supporting_perceptions {
            self.add_relationship(
                perception_id,
                *supporting_id,
                PerceptionRelationType::Supports,
            )?;
        }

        for conflicting_id in &perception.conflicting_perceptions {
            self.add_relationship(
                perception_id,
                *conflicting_id,
                PerceptionRelationType::Conflicts,
            )?;
        }

        self.perceptions.insert(perception_id, perception);
        Ok(())
    }

    /// Get a perception by ID
    pub fn get_perception(&self, perception_id: PerceptionId) -> Option<&PerceptionView> {
        self.perceptions.get(&perception_id)
    }

    /// Update perception confidence based on new evidence
    pub fn update_confidence(
        &mut self,
        perception_id: PerceptionId,
        evidence: PerceptionEvidence,
    ) -> Result<()> {
        if let Some(perception) = self.perceptions.get_mut(&perception_id) {
            let old_confidence = perception.confidence.value();

            // Simple confidence update algorithm
            // In practice, this would be more sophisticated
            let evidence_impact = evidence.weight * 0.1; // Scale evidence impact
            let new_confidence = match evidence.source {
                EvidenceSource::Observation | EvidenceSource::External => {
                    (old_confidence + evidence_impact).min(1.0)
                }
                EvidenceSource::Reasoning => {
                    old_confidence + (evidence_impact * 0.7) // Reasoning has less direct impact
                }
                EvidenceSource::Historical => {
                    old_confidence + (evidence_impact * 0.5) // Historical data has moderate impact
                }
                EvidenceSource::Consensus => {
                    old_confidence + (evidence_impact * 0.8) // Consensus has high impact
                }
                EvidenceSource::Expert => {
                    old_confidence + (evidence_impact * 0.9) // Expert judgment has very high impact
                }
            };

            perception.confidence = PerceptionConfidence::new(new_confidence)?;
            perception.evidence.push(evidence);
            perception.updated_at = Utc::now();

            Ok(())
        } else {
            Err(
                CasialError::PerceptionLock(format!("Perception {} not found", perception_id.0))
                    .into(),
            )
        }
    }

    /// Attempt to acquire a perception lock
    pub fn acquire_lock(
        &mut self,
        perception_id: PerceptionId,
        session_id: uuid::Uuid,
        lock_type: PerceptionLockType,
        duration_seconds: u64,
    ) -> Result<bool> {
        // Check if already locked
        if let Some(existing_lock) = self.active_locks.get(&perception_id) {
            if existing_lock.expires_at > Utc::now() {
                match (&existing_lock.lock_type, &lock_type) {
                    (PerceptionLockType::Exclusive, _) => return Ok(false),
                    (PerceptionLockType::Shared, PerceptionLockType::Exclusive) => {
                        return Ok(false)
                    }
                    (PerceptionLockType::Shared, PerceptionLockType::Shared) => {
                        // Allow shared locks to coexist
                    }
                    (PerceptionLockType::Advisory, _) | (_, PerceptionLockType::Advisory) => {
                        // Advisory locks don't block
                    }
                }
            }
        }

        // Acquire the lock
        let lock = PerceptionLock {
            perception_id,
            locked_by: session_id,
            locked_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::seconds(duration_seconds as i64),
            lock_type,
        };

        self.active_locks.insert(perception_id, lock);
        Ok(true)
    }

    /// Release a perception lock
    pub fn release_lock(
        &mut self,
        perception_id: PerceptionId,
        session_id: uuid::Uuid,
    ) -> Result<()> {
        if let Some(lock) = self.active_locks.get(&perception_id) {
            if lock.locked_by == session_id {
                self.active_locks.remove(&perception_id);
                Ok(())
            } else {
                Err(CasialError::PerceptionLock(format!(
                    "Lock not owned by session {}",
                    session_id
                ))
                .into())
            }
        } else {
            Err(CasialError::PerceptionLock(format!(
                "No lock found for perception {}",
                perception_id.0
            ))
            .into())
        }
    }

    /// Clean up expired locks
    pub fn cleanup_expired_locks(&mut self) -> usize {
        let now = Utc::now();
        let expired_locks: Vec<PerceptionId> = self
            .active_locks
            .iter()
            .filter(|(_, lock)| lock.expires_at <= now)
            .map(|(id, _)| *id)
            .collect();

        let count = expired_locks.len();
        for perception_id in expired_locks {
            self.active_locks.remove(&perception_id);
        }

        count
    }

    /// Add a relationship between two perceptions
    pub fn add_relationship(
        &mut self,
        from_perception: PerceptionId,
        to_perception: PerceptionId,
        relationship_type: PerceptionRelationType,
    ) -> Result<()> {
        self.perception_relationships
            .entry(from_perception)
            .or_default()
            .insert(to_perception);

        // Add reverse relationship if symmetric
        match relationship_type {
            PerceptionRelationType::Supports => {
                self.perception_relationships
                    .entry(to_perception)
                    .or_default()
                    .insert(from_perception);
            }
            PerceptionRelationType::Conflicts => {
                self.perception_relationships
                    .entry(to_perception)
                    .or_default()
                    .insert(from_perception);
            }
            PerceptionRelationType::DependsOn => {
                // Asymmetric relationship - don't add reverse
            }
            PerceptionRelationType::Enhances => {
                // Can be asymmetric
            }
        }

        Ok(())
    }

    /// Find perceptions that conflict with a given perception
    pub fn find_conflicts(&self, perception_id: PerceptionId) -> Vec<PerceptionId> {
        if let Some(perception) = self.perceptions.get(&perception_id) {
            perception.conflicting_perceptions.clone()
        } else {
            Vec::new()
        }
    }

    /// Find perceptions that support a given perception
    pub fn find_supporters(&self, perception_id: PerceptionId) -> Vec<PerceptionId> {
        if let Some(perception) = self.perceptions.get(&perception_id) {
            perception.supporting_perceptions.clone()
        } else {
            Vec::new()
        }
    }

    /// Get all perceptions with confidence above a threshold
    pub fn get_high_confidence_perceptions(&self, threshold: f64) -> Vec<PerceptionId> {
        self.perceptions
            .iter()
            .filter(|(_, perception)| perception.confidence.value() >= threshold)
            .map(|(id, _)| *id)
            .collect()
    }

    /// Get statistics about perception states
    pub fn get_statistics(&self) -> PerceptionManagerStats {
        let total_perceptions = self.perceptions.len();
        let active_locks = self.active_locks.len();

        let avg_confidence = if total_perceptions > 0 {
            self.perceptions
                .values()
                .map(|p| p.confidence.value())
                .sum::<f64>()
                / total_perceptions as f64
        } else {
            0.0
        };

        let perception_types: AHashMap<String, usize> =
            self.perceptions
                .values()
                .fold(AHashMap::new(), |mut acc, perception| {
                    let type_name = format!("{:?}", perception.perception_type);
                    *acc.entry(type_name).or_insert(0) += 1;
                    acc
                });

        PerceptionManagerStats {
            total_perceptions,
            active_locks,
            average_confidence: avg_confidence,
            perception_types,
        }
    }
}

/// Types of relationships between perceptions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerceptionRelationType {
    /// One perception supports another
    Supports,
    /// Perceptions are in conflict
    Conflicts,
    /// One perception depends on another
    DependsOn,
    /// One perception enhances another
    Enhances,
}

/// Statistics for perception manager monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerceptionManagerStats {
    pub total_perceptions: usize,
    pub active_locks: usize,
    pub average_confidence: f64,
    pub perception_types: AHashMap<String, usize>,
}

impl Default for PerceptionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perception_confidence() {
        let confidence = PerceptionConfidence::new(0.8).unwrap();
        assert!(confidence.is_high());
        assert!(!confidence.is_low());
        assert!(!confidence.is_uncertain());
    }

    #[test]
    fn test_perception_manager() {
        let mut manager = PerceptionManager::new();

        let perception = PerceptionView {
            id: PerceptionId::new(),
            name: "Test Perception".to_string(),
            description: "A test perception".to_string(),
            perception_type: PerceptionType::Human,
            confidence: PerceptionConfidence::new(0.7).unwrap(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            tags: vec!["test".to_string()],
            metadata: AHashMap::new(),
            supporting_perceptions: vec![],
            conflicting_perceptions: vec![],
            evidence: vec![],
        };

        let perception_id = perception.id;
        manager.register_perception(perception).unwrap();

        assert!(manager.get_perception(perception_id).is_some());
    }

    #[test]
    fn test_perception_locking() {
        let mut manager = PerceptionManager::new();
        let perception_id = PerceptionId::new();
        let session_id = uuid::Uuid::new_v4();

        let lock_acquired = manager
            .acquire_lock(perception_id, session_id, PerceptionLockType::Exclusive, 60)
            .unwrap();

        assert!(lock_acquired);

        manager.release_lock(perception_id, session_id).unwrap();
    }
}
